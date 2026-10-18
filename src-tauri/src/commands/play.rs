//! Bascule de compte et lancement des jeux, ajout de compte via le Riot Client.
//!
//! Chaque compte possède sa propre paire de jetons RSO (voir `riot::session`). Déroulé d'un « Jouer » :
//! 1. Le client tourne, authentifié sur le bon compte → lancement direct via l'API du client.
//! 2. Sinon : fermeture complète du client, écriture des jetons du compte cible dans la session
//!    mémorisée du client (ou écran de connexion si on n'en a pas), démarrage du client.
//! 3. Vérification : le client doit s'authentifier sur le compte cible (contrôle du PUUID).
//!    Jeton expiré → l'utilisateur se connecte dans la fenêtre officielle.
//! 4. Le client a consommé notre jeton : on en émet un nouveau pour la prochaine bascule.
//! 5. Lancement du jeu via l'API locale du client (comme le bouton Jouer officiel).

use serde::{Deserialize, Serialize};
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_notification::NotificationExt;
use zeroize::Zeroizing;

use super::accounts::{changed, new_account};
use crate::configs;
use crate::error::{AppError, AppResult};
use crate::offline::OfflineProxy;
use crate::riot::rso_login::{LoginSession, Outcome};
use crate::riot::session::RsoTokens;
use crate::riot::{captcha, identity, launch, lcu, paths, process, redeem, rso_login, session, RiotStatus};
use crate::state::AppState;
use crate::util::{log, now_ms};
use crate::vault::model::Account;

/// Délai pendant lequel des jetons restaurés doivent authentifier le client.
const AUTH_TIMEOUT: Duration = Duration::from_secs(30);
/// Délai laissé à l'utilisateur pour se connecter quand les jetons ont expiré.
const LOGIN_TIMEOUT: Duration = Duration::from_secs(300);

#[derive(Deserialize)]
pub struct Notice {
    pub title: String,
    pub body: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayOutcome {
    pub switched: bool,
}

fn set_busy(app: &AppHandle, state: &AppState, busy: Option<&str>) {
    let snapshot = {
        let mut s = state.status.lock().unwrap();
        s.busy = busy.map(str::to_string);
        s.clone()
    };
    let _ = app.emit("riot-status", snapshot);
}

/// Étape de progression affichée dans l'overlay de lancement (clé i18n côté UI).
fn emit_step(app: &AppHandle, step: &str) {
    let _ = app.emit("play-step", step);
    log(format!("étape : {step}"));
}

fn normalize_game(game: &str) -> AppResult<&'static str> {
    match game {
        "lol" => Ok("lol"),
        "tft" => Ok("tft"),
        "val" => Ok("val"),
        _ => Err(AppError::new("invalid", "game")),
    }
}

fn find_account_id(state: &AppState, id: &identity::ActiveIdentity) -> Option<String> {
    state
        .with_vault(|v| {
            Ok(v.data
                .accounts
                .iter()
                .find(|a| a.matches_identity(id.puuid.as_deref(), &id.game_name, &id.tag_line))
                .map(|a| a.id.clone()))
        })
        .ok()
        .flatten()
}

/// Identité du compte authentifié dans le client (None si déconnecté ou client fermé).
async fn signed_in_identity() -> Option<identity::ActiveIdentity> {
    if launch::authenticated().await {
        identity::active().await
    } else {
        None
    }
}

/// Jetons stockés d'un compte (None si le compte n'en a pas encore).
fn stored_tokens(state: &AppState, account_id: &str) -> AppResult<Option<RsoTokens>> {
    let secrets = state.with_vault(|v| {
        let idx = v.index_of(account_id)?;
        v.open_secrets(&v.data.accounts[idx])
    })?;
    Ok(match (&secrets.refresh_token, &secrets.id_token) {
        (Some(r), Some(i)) => Some(RsoTokens {
            refresh_token: Zeroizing::new(r.clone()),
            id_token: Zeroizing::new(i.clone()),
        }),
        _ => None,
    })
}

/// Scelle une paire de jetons dans le coffre pour ce compte.
fn save_tokens(state: &AppState, account_id: &str, tokens: &RsoTokens) -> AppResult<()> {
    let puuid = tokens.puuid();
    state.with_vault(|v| {
        let idx = v.index_of(account_id)?;
        if let (Some(known), Some(p)) = (v.data.accounts[idx].puuid.as_deref(), puuid.as_deref()) {
            if known != p {
                return Err(AppError::code("identity_mismatch"));
            }
        }
        let (r, i) = (
            tokens.refresh_token.to_string(),
            tokens.id_token.to_string(),
        );
        v.update_secrets(account_id, |s| {
            s.refresh_token = Some(r);
            s.id_token = Some(i);
        })?;
        let a = &mut v.data.accounts[idx];
        if a.puuid.is_none() {
            a.puuid = puuid;
        }
        a.session_captured_at = Some(now_ms());
        a.has_tokens = true;
        v.save()
    })
}

/// Émet une nouvelle paire de jetons pour le compte connecté au client et la scelle.
/// Réessaie quelques secondes : juste après la connexion, le client n'est pas toujours prêt.
async fn refresh_tokens(state: &AppState, account_id: &str) -> AppResult<()> {
    let mut last = AppError::code("riot_not_running");
    for _ in 0..6 {
        if let Some(api) = identity::riot_client_api() {
            match session::mint_tokens(&api).await {
                Ok(tokens) => {
                    save_tokens(state, account_id, &tokens)?;
                    log("session: nouveaux jetons émis et enregistrés");
                    return Ok(());
                }
                Err(e) => last = e,
            }
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
    log(format!(
        "session: émission des jetons impossible ({} {})",
        last.code, last.detail
    ));
    Err(last)
}

async fn ensure_proxy(state: &AppState, offline: bool) -> AppResult<std::sync::Arc<OfflineProxy>> {
    if let Some(p) = state.proxy() {
        p.set_offline(offline);
        return Ok(p);
    }
    let p = OfflineProxy::start(offline).await?;
    *state.offline.lock().unwrap() = Some(p.clone());
    Ok(p)
}

#[tauri::command]
pub async fn play(
    app: AppHandle,
    state: State<'_, AppState>,
    account_id: String,
    game: String,
    offline: bool,
    notice: Option<Notice>,
) -> AppResult<PlayOutcome> {
    state.touch();
    let game = normalize_game(&game)?;
    let _guard = state.busy.try_lock().map_err(|_| AppError::code("busy"))?;
    state.cancel_login.store(false, Ordering::Relaxed);
    set_busy(&app, &state, Some("switching"));
    let result = do_play(&app, &state, &account_id, game, offline).await;
    set_busy(&app, &state, None);
    if result.is_ok() && offline {
        if let Some(n) = notice {
            // Seule notification système de l'app : confirmation du mode hors ligne.
            let _ = app
                .notification()
                .builder()
                .title(n.title)
                .body(n.body)
                .show();
        }
    }
    if let Err(e) = &result {
        log(format!("play: échec {} {}", e.code, e.detail));
    }
    result
}

async fn do_play(
    app: &AppHandle,
    state: &AppState,
    account_id: &str,
    game: &'static str,
    offline: bool,
) -> AppResult<PlayOutcome> {
    let settings = state.settings();
    let exe = paths::riot_client_services(&settings.riot_client_path)
        .ok_or_else(|| AppError::code("riot_not_found"))?;
    let target: Account = state.with_vault(|v| {
        let idx = v.index_of(account_id)?;
        Ok(v.data.accounts[idx].clone())
    })?;
    let locale = match game {
        "val" => target.langs.val.clone(),
        "tft" => target
            .langs
            .tft
            .clone()
            .or_else(|| target.langs.lol.clone()),
        _ => target.langs.lol.clone(),
    }
    .filter(|l| !l.is_empty());

    let procs = process::snapshot();
    let ident = if procs.riot_client {
        signed_in_identity().await
    } else {
        None
    };
    let target_active = ident
        .as_ref()
        .is_some_and(|i| target.matches_identity(i.puuid.as_deref(), &i.game_name, &i.tag_line));
    let via_proxy = state.client_via_proxy.load(Ordering::Relaxed) && state.proxy().is_some();
    let restart = !procs.riot_client || !target_active || (offline && !via_proxy);
    log(format!(
        "play: jeu={game} hors_ligne={offline} client={} authentifié={} cible_active={target_active} redémarrage={restart}",
        procs.riot_client,
        ident.is_some()
    ));

    if restart {
        if procs.game_in_progress() && settings.block_switch_in_game {
            return Err(AppError::code("game_in_progress"));
        }
        emit_step(app, "prepare");
        if procs.any_riot() {
            emit_step(app, "close");
            launch::shutdown(Duration::from_secs(12)).await;
        }
        state.client_via_proxy.store(false, Ordering::Relaxed);

        let mut extra = Vec::new();
        if offline {
            let p = ensure_proxy(state, true).await?;
            extra = p.launch_args();
            state.client_via_proxy.store(true, Ordering::Relaxed);
        } else if let Some(p) = state.proxy() {
            p.set_offline(false);
        }

        prepare_game(
            state,
            &target,
            game,
            locale.as_deref(),
            &settings.lol_path,
            settings.auto_apply_profiles,
        );
        sign_in_target(app, state, &exe, &target, game, &extra).await?;
        emit_step(app, "launch");
    } else {
        if let Some(p) = state.proxy() {
            p.set_offline(offline);
        }
        prepare_game(
            state,
            &target,
            game,
            locale.as_deref(),
            &settings.lol_path,
            settings.auto_apply_profiles,
        );
        if stored_tokens(state, &target.id)?.is_none() {
            let _ = refresh_tokens(state, &target.id).await;
        }
    }

    launch::launch_product(&exe, game, locale.as_deref()).await?;
    record_launch(state, &target.id, game);
    after_launch(app.clone(), target.id.clone(), game);
    Ok(PlayOutcome {
        switched: !target_active,
    })
}

/// Identifiants enregistrés pour un compte (déchiffrés le temps de la connexion).
fn stored_credentials(
    state: &AppState,
    account_id: &str,
) -> AppResult<Option<(Zeroizing<String>, Zeroizing<String>)>> {
    let secrets = state.with_vault(|v| {
        let idx = v.index_of(account_id)?;
        v.open_secrets(&v.data.accounts[idx])
    })?;
    Ok(match (&secrets.username, &secrets.password) {
        (Some(u), Some(p)) if !u.is_empty() && !p.is_empty() => {
            Some((Zeroizing::new(u.clone()), Zeroizing::new(p.clone())))
        }
        _ => None,
    })
}

/// Connecte le compte cible, en trois recours : identifiants enregistrés (connexion assistée,
/// captcha et A2F résolus par l'utilisateur), sinon jetons RSO restaurés, sinon connexion manuelle.
async fn sign_in_target(
    app: &AppHandle,
    state: &AppState,
    exe: &std::path::Path,
    target: &Account,
    game: &'static str,
    extra: &[String],
) -> AppResult<()> {
    // 1) Identifiants enregistrés → connexion assistée.
    if let Some((user, pass)) = stored_credentials(state, &target.id)? {
        // Démarrer le client déconnecté AVANT la connexion (comme LAM) : il a le temps d'être prêt
        // pendant que l'utilisateur résout le captcha, et reçoit ensuite le jeton sans délai.
        session::clear_persisted()?;
        launch::start_client(exe, Some(game), extra)?;
        emit_step(app, "login");
        let token = credential_login(app, state, &user, &pass).await?;
        drop((user, pass));
        emit_step(app, "signin");
        if !launch::wait_api(Duration::from_secs(45)).await {
            return Err(AppError::new("riot_not_running", "api"));
        }
        redeem::redeem(&token, true).await?;
        verify_target(state, target).await?;
        let _ = refresh_tokens(state, &target.id).await;
        return Ok(());
    }
    // 2) Jetons RSO enregistrés → restauration directe (sans captcha).
    if let Some(tokens) = stored_tokens(state, &target.id)? {
        session::write_persisted(&tokens)?;
        launch::start_client(exe, Some(game), extra)?;
        wait_signed_in(app, state, target).await?;
        let _ = refresh_tokens(state, &target.id).await;
        return Ok(());
    }
    // 3) Rien d'enregistré → écran de connexion officiel.
    session::clear_persisted()?;
    launch::start_client(exe, Some(game), extra)?;
    wait_signed_in(app, state, target).await
}

/// Connexion au service d'authentification de Riot avec les identifiants enregistrés.
/// Renvoie un jeton de connexion à faire consommer par le client. Captcha et A2F résolus par l'utilisateur.
async fn credential_login(
    app: &AppHandle,
    state: &AppState,
    user: &str,
    pass: &str,
) -> AppResult<Zeroizing<String>> {
    let session = LoginSession::new()?;
    let challenge = session.start(true).await?;
    let captcha_token = if challenge.requires() {
        emit_step(app, "captcha");
        solve_captcha(state, &challenge).await?
    } else {
        String::new()
    };
    let mut outcome = session.complete(user, pass, true, &captcha_token).await?;
    loop {
        match outcome {
            Outcome::Success { login_token } => {
                log("login: identifiants acceptés, jeton de connexion obtenu");
                return Ok(Zeroizing::new(login_token));
            }
            Outcome::Failed { reason } => {
                return Err(AppError::new("login_failed", reason));
            }
            Outcome::Mfa { methods, email } => {
                log("login: A2F requise");
                let _ = app.emit("login-mfa", serde_json::json!({ "methods": methods, "email": email }));
                let otp = await_otp(state).await?;
                outcome = session.submit_mfa(&otp, true).await?;
            }
            Outcome::Challenge { challenge } => {
                emit_step(app, "captcha");
                let tok = solve_captcha(state, &challenge).await?;
                outcome = session.complete(user, pass, true, &tok).await?;
            }
        }
    }
}

/// Affiche la fenêtre de défi hCaptcha (résolu par l'utilisateur) et renvoie le jeton.
async fn solve_captcha(state: &AppState, challenge: &rso_login::Challenge) -> AppResult<String> {
    let site = challenge
        .site_key
        .clone()
        .ok_or_else(|| AppError::new("captcha", "pas de sitekey"))?;
    let rq = challenge.rq_data.clone().unwrap_or_default();
    let host = rso_login::CLIENTLESS_HOST.to_string();
    // Dossier WebView2 neuf à chaque appel : évite qu'un verrou résiduel bloque un nouvel essai.
    let base = state.paths.cache_dir().join("webview2");
    let _ = std::fs::remove_dir_all(&base);
    let dir = base.join(uuid::Uuid::new_v4().to_string());
    let _ = std::fs::create_dir_all(&dir);
    // Thread OS neuf à chaque appel : la fenêtre WebView2 a une file de messages propre (un WM_QUIT
    // résiduel d'un captcha précédent, sur un thread du pool réutilisé, annulait le suivant).
    let (tx, rx) = tokio::sync::oneshot::channel();
    std::thread::Builder::new()
        .name("riftswitch-captcha".into())
        .spawn(move || {
            let _ = tx.send(captcha::solve(&site, &rq, &host, &dir));
        })
        .map_err(|e| AppError::new("captcha", e.to_string()))?;
    let token = rx.await.map_err(|_| AppError::code("cancelled"))??;
    token.ok_or_else(|| AppError::code("cancelled"))
}

/// Attend le code d'A2F fourni par l'utilisateur via la commande `submit_mfa`.
async fn await_otp(state: &AppState) -> AppResult<String> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    *state.pending_otp.lock().await = Some(tx);
    match tokio::time::timeout(Duration::from_secs(300), rx).await {
        Ok(Ok(code)) => Ok(code),
        Ok(Err(_)) => Err(AppError::code("cancelled")),
        Err(_) => Err(AppError::code("timeout")),
    }
}

/// Attend que le client soit authentifié après redemption d'un jeton. Si l'identité du compte était
/// inconnue (ajouté par identifiants, sans Riot ID), on enregistre celle qui vient de se connecter.
async fn verify_target(state: &AppState, target: &Account) -> AppResult<()> {
    let known = target.puuid.is_some() || !target.game_name.is_empty();
    let start = Instant::now();
    while start.elapsed() < AUTH_TIMEOUT {
        if let Some(i) = signed_in_identity().await {
            if known {
                return if target.matches_identity(i.puuid.as_deref(), &i.game_name, &i.tag_line) {
                    log("login: client authentifié sur le compte cible");
                    Ok(())
                } else {
                    Err(AppError::code("identity_mismatch"))
                };
            }
            // Compte sans identité connue : on adopte celle qui s'est connectée.
            let _ = state.with_vault(|v| {
                let idx = v.index_of(&target.id)?;
                let a = &mut v.data.accounts[idx];
                a.observe_riot_id(&i.game_name, &i.tag_line, now_ms());
                a.puuid = i.puuid.clone();
                v.save()
            });
            log("login: compte identifié et enregistré après connexion");
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(800)).await;
    }
    Err(AppError::code("not_signed_in"))
}

/// Code d'A2F fourni par l'interface pendant une connexion.
#[tauri::command]
pub async fn submit_mfa(state: State<'_, AppState>, code: String) -> AppResult<()> {
    match state.pending_otp.lock().await.take() {
        Some(tx) => {
            let _ = tx.send(code);
            Ok(())
        }
        None => Err(AppError::code("no_mfa_pending")),
    }
}

/// Attend que le client soit authentifié sur le compte cible. Si les jetons ont expiré (ou n'existent
/// pas), l'utilisateur se connecte dans la fenêtre officielle du Riot Client.
async fn wait_signed_in(app: &AppHandle, state: &AppState, target: &Account) -> AppResult<()> {
    if !launch::wait_api(Duration::from_secs(45)).await {
        return Err(AppError::new("riot_not_running", "api"));
    }
    let matches = |i: &identity::ActiveIdentity| {
        target.matches_identity(i.puuid.as_deref(), &i.game_name, &i.tag_line)
    };

    // 1) Jetons restaurés : le client doit s'authentifier tout seul.
    let start = Instant::now();
    while start.elapsed() < AUTH_TIMEOUT {
        if let Some(i) = signed_in_identity().await {
            if matches(&i) {
                log(format!(
                    "play: client authentifié sur le compte cible en {} ms",
                    start.elapsed().as_millis()
                ));
                return Ok(());
            }
            log("play: client authentifié sur un AUTRE compte que la cible");
            return Err(AppError::code("identity_mismatch"));
        }
        tokio::time::sleep(Duration::from_millis(800)).await;
    }

    // 2) Jetons expirés ou absents : connexion dans la fenêtre officielle du Riot Client.
    log("play: jetons refusés ou absents → attente de connexion dans le Riot Client");
    set_busy(app, state, Some("waiting_login"));
    let _ = app.emit("session-expired", target.label.clone());
    let start = Instant::now();
    while start.elapsed() < LOGIN_TIMEOUT {
        if state.cancel_login.load(Ordering::Relaxed) {
            return Err(AppError::code("cancelled"));
        }
        if let Some(i) = signed_in_identity().await {
            if !matches(&i) {
                return Err(AppError::code("identity_mismatch"));
            }
            log("play: connexion manuelle réussie");
            set_busy(app, state, Some("switching"));
            return Ok(());
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    Err(AppError::code("timeout"))
}

/// Avant lancement : langue du jeu et profil de config Valorant (fichiers).
fn prepare_game(
    state: &AppState,
    target: &Account,
    game: &str,
    locale: Option<&str>,
    lol_custom: &str,
    auto_profiles: bool,
) {
    if let Some(l) = locale {
        paths::set_game_locale(game, l, lol_custom);
    }
    if auto_profiles && game == "val" {
        if let Some(pid) = target.profiles.val.as_deref() {
            if let (Ok(p), Some(dir)) = (
                state.profiles.get(pid),
                configs::val_dir_for(target.puuid.as_deref()),
            ) {
                let r = configs::apply_val(&p, &dir);
                log(format!("play: profil Valorant appliqué ok={}", r.is_ok()));
            }
        }
    }
}

fn record_launch(state: &AppState, account_id: &str, game: &str) {
    let _ = state.with_vault(|v| {
        let idx = v.index_of(account_id)?;
        let a = &mut v.data.accounts[idx];
        a.last_used_at = Some(now_ms());
        a.last_game = Some(game.to_string());
        *a.playtime.launches.entry(game.to_string()).or_default() += 1;
        v.data.last_active_account = Some(account_id.to_string());
        v.save()
    });
    let mut s = state.settings();
    if s.last_game != game {
        s.last_game = game.to_string();
        let _ = s.save(&state.paths);
        *state.settings.lock().unwrap() = s;
    }
    state.force_sync.store(true, Ordering::Relaxed);
}

/// Après lancement : application du profil LoL via le client une fois connecté.
fn after_launch(app: AppHandle, account_id: String, game: &'static str) {
    changed(&app);
    if game == "val" {
        return;
    }
    tauri::async_runtime::spawn(async move {
        let state = app.state::<AppState>();
        let settings = state.settings();
        if !settings.auto_apply_profiles {
            return;
        }
        let profile_id = state
            .with_vault(|v| {
                Ok(v.data.accounts[v.index_of(&account_id)?]
                    .profiles
                    .lol
                    .clone())
            })
            .ok()
            .flatten();
        let Some(pid) = profile_id else { return };
        let Ok(profile) = state.profiles.get(&pid) else {
            return;
        };
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(240) {
            if let Some(api) = lcu::api(&settings.lol_path) {
                if lcu::logged_in(&api).await {
                    let r = configs::apply_lol(&profile, Some(&api), &settings.lol_path).await;
                    log(format!(
                        "play: profil LoL appliqué après connexion ok={}",
                        r.is_ok()
                    ));
                    return;
                }
            }
            tokio::time::sleep(Duration::from_secs(3)).await;
        }
    });
}

/// Lancement depuis la zone de notification ou la ligne de commande (`--play <id> <jeu>`).
pub fn play_external(app: AppHandle, account_id: String, game: String) {
    tauri::async_runtime::spawn(async move {
        let state = app.state::<AppState>();
        let Ok(game) = normalize_game(&game) else {
            return;
        };
        let Ok(_guard) = state.busy.try_lock() else {
            return;
        };
        state.cancel_login.store(false, Ordering::Relaxed);
        set_busy(&app, &state, Some("switching"));
        let r = do_play(&app, &state, &account_id, game, false).await;
        set_busy(&app, &state, None);
        if let Err(e) = r {
            log(format!("play (externe): échec {}", e.code));
            let _ = app.emit("play-error", e);
        }
    });
}

#[tauri::command]
pub fn set_offline(state: State<'_, AppState>, offline: bool) -> AppResult<bool> {
    match state.proxy() {
        Some(p) if state.client_via_proxy.load(Ordering::Relaxed) => {
            p.set_offline(offline);
            Ok(true)
        }
        // Le client ne passe pas par le proxy : il faudra relancer via « Jouer » en hors ligne.
        _ => Ok(false),
    }
}

#[tauri::command]
pub fn riot_status(state: State<'_, AppState>) -> RiotStatus {
    state.status.lock().unwrap().clone()
}

#[tauri::command]
pub async fn quit_riot(state: State<'_, AppState>) -> AppResult<()> {
    let _guard = state.busy.try_lock().map_err(|_| AppError::code("busy"))?;
    if process::snapshot().game_in_progress() {
        return Err(AppError::code("game_in_progress"));
    }
    launch::shutdown(Duration::from_secs(12)).await;
    Ok(())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureOutcome {
    pub account_id: String,
    pub created: bool,
}

/// Crée (ou retrouve) le compte du joueur connecté, puis lui émet et scelle sa paire de jetons.
async fn save_signed_in_account(
    state: &AppState,
    target_id: Option<&str>,
    ident: &identity::ActiveIdentity,
) -> AppResult<CaptureOutcome> {
    let existing = match target_id {
        Some(id) => Some(id.to_string()),
        None => find_account_id(state, ident),
    };
    let fresh = new_account(state);
    let outcome = state.with_vault(|v| {
        let (idx, created) = match existing {
            Some(id) => {
                let idx = v.index_of(&id)?;
                let a = &v.data.accounts[idx];
                let known = a.puuid.is_some() || !a.game_name.is_empty();
                if known
                    && !a.matches_identity(
                        ident.puuid.as_deref(),
                        &ident.game_name,
                        &ident.tag_line,
                    )
                {
                    return Err(AppError::code("identity_mismatch"));
                }
                (idx, false)
            }
            None => {
                v.data.accounts.push(fresh);
                (v.data.accounts.len() - 1, true)
            }
        };
        let now = now_ms();
        let a = &mut v.data.accounts[idx];
        a.observe_riot_id(&ident.game_name, &ident.tag_line, now);
        if ident.puuid.is_some() {
            a.puuid = ident.puuid.clone();
        }
        if a.label.is_empty() {
            a.label = if a.game_name.is_empty() {
                "Riot".into()
            } else {
                a.game_name.clone()
            };
        }
        let id = a.id.clone();
        v.data.last_active_account = Some(id.clone());
        v.save()?;
        Ok(CaptureOutcome {
            account_id: id,
            created,
        })
    })?;
    refresh_tokens(state, &outcome.account_id).await?;
    Ok(outcome)
}

/// « Ajouter le compte actuellement connecté au Riot Client » (ou renouveler les jetons d'un compte).
#[tauri::command]
pub async fn capture_current(
    app: AppHandle,
    state: State<'_, AppState>,
    target_id: Option<String>,
) -> AppResult<CaptureOutcome> {
    state.touch();
    let _guard = state.busy.try_lock().map_err(|_| AppError::code("busy"))?;
    let ident = signed_in_identity()
        .await
        .ok_or_else(|| AppError::code("not_signed_in"))?;
    let out = save_signed_in_account(&state, target_id.as_deref(), &ident).await?;
    state.force_sync.store(true, Ordering::Relaxed);
    changed(&app);
    Ok(out)
}

/// Connexion d'un nouveau compte dans la fenêtre officielle du Riot Client (identifiants et A2F
/// saisis chez Riot, jamais dans RiftSwitch), puis émission de sa paire de jetons.
#[tauri::command]
pub async fn add_via_riot_login(
    app: AppHandle,
    state: State<'_, AppState>,
) -> AppResult<CaptureOutcome> {
    state.touch();
    let settings = state.settings();
    let exe = paths::riot_client_services(&settings.riot_client_path)
        .ok_or_else(|| AppError::code("riot_not_found"))?;
    let _guard = state.busy.try_lock().map_err(|_| AppError::code("busy"))?;
    let procs = process::snapshot();
    if procs.game_in_progress() {
        return Err(AppError::code("game_in_progress"));
    }
    set_busy(&app, &state, Some("waiting_login"));
    state.cancel_login.store(false, Ordering::Relaxed);

    let result = async {
        // Le compte actuellement connecté garde ses jetons RiftSwitch (on lui en émet s'il n'en a pas).
        if procs.riot_client {
            if let Some(out) = signed_in_identity()
                .await
                .and_then(|i| find_account_id(&state, &i))
            {
                if stored_tokens(&state, &out)?.is_none() {
                    let _ = refresh_tokens(&state, &out).await;
                }
            }
        }
        if procs.any_riot() {
            launch::shutdown(Duration::from_secs(12)).await;
        }
        state.client_via_proxy.store(false, Ordering::Relaxed);
        session::clear_persisted()?;
        launch::start_client(&exe, None, &[])?;

        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(600) {
            tokio::time::sleep(Duration::from_secs(2)).await;
            if state.cancel_login.load(Ordering::Relaxed) {
                return Err(AppError::code("cancelled"));
            }
            if let Some(id) = signed_in_identity().await {
                log("ajout: connexion détectée, émission des jetons");
                return save_signed_in_account(&state, None, &id).await;
            }
        }
        Err(AppError::code("timeout"))
    }
    .await;

    set_busy(&app, &state, None);
    if result.is_ok() {
        state.force_sync.store(true, Ordering::Relaxed);
        changed(&app);
    }
    result
}

#[tauri::command]
pub fn cancel_login(state: State<'_, AppState>) {
    state.cancel_login.store(true, Ordering::Relaxed);
}
