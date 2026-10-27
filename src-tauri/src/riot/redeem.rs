//! Fait consommer un jeton de connexion Riot par le Riot Client local (d'après League-Account-Manager).
//!
//! Le jeton de connexion vient du flux `rso_login` (identifiants rejoués par l'utilisateur, captcha
//! résolu par l'utilisateur). On le remet au client déjà démarré et déconnecté :
//! 1. via le plugin `player-session-lifecycle` (PUT du jeton), ce que fait le client lui-même ;
//! 2. sinon, repli par le gestionnaire de protocole `riotclient://auth/v1/<jeton>` (le client le
//!    consomme nativement). Puis on marque la session « rester connecté » et on accepte le CLUF.
//!
//! Le jeton est à usage unique ; aucune valeur n'est journalisée.

use serde_json::{json, Value};
use std::time::{Duration, Instant};
use zeroize::Zeroizing;

use super::{identity, local_api::LocalApi};
use crate::error::{AppError, AppResult};
use crate::util::log;

const AUTH_TIMEOUT: Duration = Duration::from_secs(90);
const POLL: Duration = Duration::from_millis(250);

/// Attend que le plugin rso-auth du client soit prêt (configuration RSO chargée).
async fn wait_ready(api: &LocalApi) -> bool {
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(60) {
        if let Ok(v) = api.get("/rso-auth/configuration/v3/ready-state").await {
            if v.get("ready").and_then(|x| x.as_bool()).unwrap_or(false) {
                return true;
            }
        }
        tokio::time::sleep(POLL).await;
    }
    false
}

/// Remet le jeton de connexion au client déjà démarré, puis rend la session fiable et accepte le CLUF.
pub async fn redeem(login_token: &str, remember: bool) -> AppResult<()> {
    let token = Zeroizing::new(login_token.to_string());
    let api = identity::riot_client_api().ok_or_else(|| AppError::code("riot_not_running"))?;
    if !wait_ready(&api).await {
        log("redeem: rso-auth pas prêt dans les temps");
        return Err(AppError::new("redeem", "rso pas prêt"));
    }

    // 1) Plugin de cycle de vie de session (ce que fait le gestionnaire de protocole du client).
    //    On ne fixe PAS la stratégie de connexion : cela ferait chercher au client des identifiants
    //    en ligne de commande qu'il n'a pas, et le jeton serait rejeté.
    let body = json!({ "loginToken": &*token, "remember": remember });
    let mut authed = match api
        .put("/player-session-lifecycle/v1/login-token", &body)
        .await
    {
        Ok(_) => {
            log("redeem: jeton accepté par le plugin de session");
            wait_authenticated(&api).await
        }
        Err(e) => {
            log(format!(
                "redeem: lifecycle a refusé ({} {})",
                e.code, e.detail
            ));
            false
        }
    };

    // 2) Repli : remettre le jeton via le gestionnaire de protocole du client (natif, très fiable).
    if !authed {
        let scheme = uri_scheme(&api).await;
        let url = format!("{scheme}://auth/v1/{}", *token);
        log(format!(
            "redeem: repli par le gestionnaire de protocole ({scheme}://auth/v1/…)"
        ));
        if open_url(&url).is_ok() {
            authed = wait_authenticated(&api).await;
        }
    }

    if !authed {
        return Err(AppError::code("redeem_failed"));
    }
    log("redeem: session authentifiée");

    // 3) Rendre la session fiable pour qu'elle survive à un redémarrage (« rester connecté »).
    if remember {
        let trust = json!({ "clientId": "riot-client", "trustLevels": ["always_trusted"] });
        let r = api.post("/rso-auth/v2/authorizations", &trust).await;
        log(format!("redeem: autorisation fiable ok={}", r.is_ok()));
    }

    accept_eula(&api).await;
    Ok(())
}

/// Demande au client le schéma de son gestionnaire de protocole (défaut : `riotclient`).
async fn uri_scheme(api: &LocalApi) -> String {
    if let Ok(v) = api
        .post("/riot-client-app-command/v1/uri-handler", &Value::Null)
        .await
    {
        if let Some(obj) = v.as_object() {
            if let Some(s) = obj.values().find_map(|x| x.as_str()) {
                if !s.is_empty() {
                    return s.to_string();
                }
            }
        }
    }
    "riotclient".to_string()
}

async fn wait_authenticated(api: &LocalApi) -> bool {
    let start = Instant::now();
    while start.elapsed() < AUTH_TIMEOUT {
        if let Ok(v) = api.get("/player-session-lifecycle/v1/session").await {
            match v.get("type").and_then(|x| x.as_str()) {
                Some("authenticated") => return true,
                Some("error") => {
                    log("redeem: le plugin de session signale une erreur");
                    return false;
                }
                _ => {}
            }
        }
        // Repli : l'autorisation RSO existe-t-elle déjà (sujet présent) ?
        if api
            .get("/rso-auth/v1/authorization")
            .await
            .map(|v| v.get("subject").and_then(|x| x.as_str()).is_some())
            .unwrap_or(false)
        {
            return true;
        }
        tokio::time::sleep(POLL).await;
    }
    false
}

async fn accept_eula(api: &LocalApi) {
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(30) {
        match api.get("/eula/v1/agreement/acceptance").await {
            Ok(Value::String(s)) if s == "Accepted" => return,
            Ok(Value::String(s)) if s == "AcceptanceRequired" => {
                let _ = api.put("/eula/v1/agreement/acceptance", &Value::Null).await;
            }
            _ => {}
        }
        tokio::time::sleep(POLL).await;
    }
}

/// Ouvre une URL de protocole (ex. `riotclient://auth/v1/<jeton>`) via le shell Windows.
#[cfg(windows)]
fn open_url(url: &str) -> AppResult<()> {
    use windows::core::HSTRING;
    use windows::Win32::UI::Shell::ShellExecuteW;
    use windows::Win32::UI::WindowsAndMessaging::SW_HIDE;
    let op = HSTRING::from("open");
    let file = HSTRING::from(url);
    let r = unsafe {
        ShellExecuteW(
            None,
            windows::core::PCWSTR(op.as_ptr()),
            windows::core::PCWSTR(file.as_ptr()),
            windows::core::PCWSTR::null(),
            windows::core::PCWSTR::null(),
            SW_HIDE,
        )
    };
    // ShellExecuteW renvoie > 32 en cas de succès.
    if r.0 as usize > 32 {
        Ok(())
    } else {
        Err(AppError::new("redeem", "ouverture du protocole échouée"))
    }
}

#[cfg(not(windows))]
fn open_url(_url: &str) -> AppResult<()> {
    Err(AppError::code("redeem_failed"))
}
