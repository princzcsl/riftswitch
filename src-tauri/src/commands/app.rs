//! Commandes générales : démarrage, coffre, réglages, réinitialisation, sauvegardes, fenêtre.

use serde::Serialize;
use std::sync::atomic::Ordering;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, State};

use crate::error::{AppError, AppResult};
use crate::settings::Settings;
use crate::state::AppState;
use crate::util::log;
use crate::vault::Vault;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Bootstrap {
    pub vault_exists: bool,
    pub vault_mode: Option<String>,
    pub unlocked: bool,
    pub settings: Settings,
    pub portable: bool,
    pub version: String,
    pub os: String,
    pub data_dir: String,
}

#[tauri::command]
pub fn get_bootstrap(app: AppHandle, state: State<'_, AppState>) -> Bootstrap {
    let path = state.paths.vault();
    Bootstrap {
        vault_exists: Vault::exists(&path),
        vault_mode: Vault::peek_mode(&path),
        unlocked: state.is_unlocked(),
        settings: state.settings(),
        portable: state.paths.portable,
        version: app.package_info().version.to_string(),
        os: std::env::consts::OS.to_string(),
        data_dir: state.paths.root.display().to_string(),
    }
}

#[tauri::command]
pub fn save_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    settings: Settings,
) -> AppResult<Settings> {
    let mut s = settings.sanitized();
    s.telemetry_last_day = state.settings().telemetry_last_day;
    if s.telemetry && s.telemetry_install_id.is_empty() {
        s.telemetry_install_id = uuid::Uuid::new_v4().to_string();
    }
    s.save(&state.paths)?;
    *state.settings.lock().unwrap() = s.clone();
    crate::tray::refresh(&app);
    Ok(s)
}

#[tauri::command]
pub fn touch(state: State<'_, AppState>) {
    state.touch();
}

#[tauri::command]
pub async fn setup_vault(
    app: AppHandle,
    state: State<'_, AppState>,
    password: Option<String>,
) -> AppResult<()> {
    let path = state.paths.vault();
    if Vault::exists(&path) {
        return Err(AppError::code("vault_exists"));
    }
    if password.is_none() && state.paths.portable {
        return Err(AppError::code("portable_needs_password"));
    }
    // Argon2 est volontairement lent : hors du thread principal.
    let vault =
        tauri::async_runtime::spawn_blocking(move || Vault::create(&path, password.as_deref()))
            .await
            .map_err(|e| AppError::new("internal", e.to_string()))??;
    *state.vault.lock().unwrap() = Some(vault);
    state.touch();
    log("vault: créé");
    crate::tray::refresh(&app);
    Ok(())
}

#[tauri::command]
pub async fn unlock(
    app: AppHandle,
    state: State<'_, AppState>,
    password: Option<String>,
) -> AppResult<()> {
    // Anti force brute : délai croissant après chaque échec (max 8 s).
    let fails = state.failed_unlocks.load(Ordering::Relaxed);
    if fails > 2 {
        tokio::time::sleep(Duration::from_millis((250u64 << fails.min(5)).min(8000))).await;
    }
    let path = state.paths.vault();
    let result =
        tauri::async_runtime::spawn_blocking(move || Vault::unlock(&path, password.as_deref()))
            .await
            .map_err(|e| AppError::new("internal", e.to_string()))?;
    match result {
        Ok(mut vault) => {
            state.failed_unlocks.store(0, Ordering::Relaxed);
            // Temps de jeu compté pendant le verrouillage.
            let pending: Vec<_> = std::mem::take(&mut *state.pending_playtime.lock().unwrap());
            if !pending.is_empty() {
                for (id, game, secs) in pending {
                    crate::monitor::credit_playtime(&mut vault, &id, &game, secs);
                }
                let _ = vault.save();
            }
            *state.vault.lock().unwrap() = Some(vault);
            state.touch();
            state.force_sync.store(true, Ordering::Relaxed);
            crate::tray::refresh(&app);
            Ok(())
        }
        Err(e) => {
            state.failed_unlocks.fetch_add(1, Ordering::Relaxed);
            Err(e)
        }
    }
}

#[tauri::command]
pub fn lock(app: AppHandle, state: State<'_, AppState>) -> AppResult<()> {
    do_lock(&app, &state);
    Ok(())
}

pub fn do_lock(app: &AppHandle, state: &AppState) {
    // Le drop du coffre efface la clé (Zeroizing).
    *state.vault.lock().unwrap() = None;
    let _ = app.emit("vault-locked", ());
    crate::tray::refresh(app);
}

#[tauri::command]
pub async fn change_master_password(
    state: State<'_, AppState>,
    new_password: Option<String>,
) -> AppResult<()> {
    if new_password.is_none() && state.paths.portable {
        return Err(AppError::code("portable_needs_password"));
    }
    // On sort le coffre le temps de la dérivation (lente), puis on le remet.
    let mut vault = state
        .vault
        .lock()
        .unwrap()
        .take()
        .ok_or_else(|| AppError::code("vault_locked"))?;
    let (vault, result) = tauri::async_runtime::spawn_blocking(move || {
        let r = vault.change_protection(new_password.as_deref());
        (vault, r)
    })
    .await
    .map_err(|e| AppError::new("internal", e.to_string()))?;
    *state.vault.lock().unwrap() = Some(vault);
    result
}

/// Efface toutes les données : coffre, clé système, réglages, profils, cache, journal.
#[tauri::command]
pub fn reset_app(app: AppHandle, state: State<'_, AppState>) -> AppResult<()> {
    *state.vault.lock().unwrap() = None;
    Vault::destroy(&state.paths.vault());
    let root = &state.paths.root;
    for p in [state.paths.settings(), state.paths.log()] {
        let _ = std::fs::remove_file(p);
    }
    let _ = std::fs::remove_dir_all(state.paths.profiles_dir());
    let _ = std::fs::remove_dir_all(state.paths.cache_dir());
    // Sauvegardes de migration éventuelles.
    if let Ok(entries) = std::fs::read_dir(root) {
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if name.starts_with("vault.") {
                let _ = std::fs::remove_file(e.path());
            }
        }
    }
    *state.settings.lock().unwrap() = Settings::default();
    log("app: réinitialisée");
    crate::tray::refresh(&app);
    Ok(())
}

#[tauri::command]
pub async fn export_backup(
    state: State<'_, AppState>,
    path: String,
    passphrase: String,
) -> AppResult<()> {
    let blob = {
        let guard = state.vault.lock().unwrap();
        let vault = guard
            .as_ref()
            .ok_or_else(|| AppError::code("vault_locked"))?;
        // Export lent (Argon2) mais coffre petit : on garde le verrou le temps de chiffrer.
        vault.export(&passphrase)?
    };
    crate::util::atomic_write(std::path::Path::new(&path), &blob, false)?;
    Ok(())
}

#[tauri::command]
pub async fn import_backup(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
    passphrase: String,
) -> AppResult<usize> {
    let bytes = std::fs::read(&path)?;
    let added = state.with_vault(|v| v.import(&bytes, &passphrase))?;
    let _ = app.emit("accounts-changed", ());
    crate::tray::refresh(&app);
    Ok(added)
}

/// Ce qui serait envoyé si la télémétrie était activée (écran « Voir ce qui est envoyé »).
#[tauri::command]
pub fn telemetry_preview(app: AppHandle, state: State<'_, AppState>) -> serde_json::Value {
    build_preview(&app, &state)
}

pub fn build_preview(app: &AppHandle, state: &AppState) -> serde_json::Value {
    let s = state.settings();
    let (count, launches, hours) = state
        .with_vault(|v| {
            let n = v.data.accounts.len();
            let secs: u64 = v
                .data
                .accounts
                .iter()
                .map(|a| a.playtime.lol + a.playtime.tft + a.playtime.val)
                .sum();
            let mut l = std::collections::BTreeMap::<String, u64>::new();
            for a in &v.data.accounts {
                for (g, c) in &a.playtime.launches {
                    *l.entry(g.clone()).or_default() += c;
                }
            }
            Ok((n, l, (secs as f64 / 3600.0).round() as u64))
        })
        .unwrap_or_default();
    let bucket = match count {
        0..=1 => "1",
        2..=5 => "2-5",
        6..=10 => "6-10",
        _ => "10+",
    };
    serde_json::json!({
        "installId": if s.telemetry_install_id.is_empty() { "(généré à l'activation)".to_string() } else { s.telemetry_install_id },
        "accountsBucket": bucket,
        "launches": launches,
        "hoursPlayedRounded": hours,
        "version": app.package_info().version.to_string(),
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "language": s.language,
        "theme": s.theme,
    })
}

// ── Fenêtre ─────────────────────────────────────────────────────────────────

/// Fermer : soit on quitte, soit on détruit la fenêtre (la webview libère sa mémoire) et l'app
/// reste dans la zone de notification avec une empreinte minimale.
#[tauri::command]
pub fn close_window(app: AppHandle, state: State<'_, AppState>) {
    if state.settings().close_to_tray {
        if let Some(w) = app.get_webview_window("main") {
            state.window_visible.store(false, Ordering::Relaxed);
            let _ = w.destroy();
        }
    } else {
        app.exit(0);
    }
}

#[tauri::command]
pub fn quit_app(app: AppHandle) {
    app.exit(0);
}

#[tauri::command]
pub fn open_data_dir(state: State<'_, AppState>) -> AppResult<()> {
    let dir = state.paths.root.clone();
    #[cfg(windows)]
    let _ = std::process::Command::new("explorer").arg(dir).spawn();
    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open").arg(dir).spawn();
    Ok(())
}
