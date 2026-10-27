//! Commandes des profils de config (touches, interface, son, graphismes) et des runes.

use serde_json::Value;
use tauri::{AppHandle, State};

use super::accounts::changed;
use crate::configs::{self, ConfigProfile, LolConfig, ValDir};
use crate::error::{AppError, AppResult};
use crate::riot::lcu;
use crate::runes;
use crate::state::AppState;
use crate::util::{log, now_ms};
use crate::vault::model::RunePage;

#[tauri::command]
pub fn list_profiles(state: State<'_, AppState>) -> Vec<ConfigProfile> {
    state.touch();
    state.profiles.list()
}

#[tauri::command]
pub fn get_profile(state: State<'_, AppState>, id: String) -> AppResult<ConfigProfile> {
    state.profiles.get(&id)
}

#[tauri::command]
pub fn save_profile(
    state: State<'_, AppState>,
    profile: ConfigProfile,
) -> AppResult<ConfigProfile> {
    state.touch();
    state.profiles.save(profile)
}

#[tauri::command]
pub fn delete_profile(app: AppHandle, state: State<'_, AppState>, id: String) -> AppResult<()> {
    state.profiles.delete(&id)?;
    // Détacher le profil des comptes qui l'utilisaient.
    let _ = state.with_vault(|v| {
        for a in v.data.accounts.iter_mut() {
            if a.profiles.lol.as_deref() == Some(id.as_str()) {
                a.profiles.lol = None;
            }
            if a.profiles.val.as_deref() == Some(id.as_str()) {
                a.profiles.val = None;
            }
        }
        v.save()
    });
    changed(&app);
    Ok(())
}

#[tauri::command]
pub fn duplicate_profile(
    state: State<'_, AppState>,
    id: String,
    name: String,
) -> AppResult<ConfigProfile> {
    let mut p = state.profiles.get(&id)?;
    p.id.clear();
    p.name = name;
    state.profiles.save(p)
}

/// Riot ID et icône d'un compte du coffre.
fn account_source(state: &AppState, id: &str) -> Option<(String, Option<i64>)> {
    state
        .with_vault(|v| {
            Ok(v.data.accounts.iter().find(|a| a.id == id).map(|a| {
                let name = if a.tag_line.is_empty() {
                    a.game_name.clone()
                } else {
                    format!("{}#{}", a.game_name, a.tag_line)
                };
                (name, a.profile_icon_id)
            }))
        })
        .ok()
        .flatten()
}

/// Avant la première application d'un autre profil sur un compte : sauvegarde de sa config
/// actuelle, une seule fois par compte et par jeu, pour pouvoir revenir en arrière.
pub(crate) fn backup_once(
    state: &AppState,
    account_id: &str,
    applied: &ConfigProfile,
    lol: Option<LolConfig>,
    val: Option<configs::ValConfig>,
) {
    if applied.source_account == account_id || applied.source == "backup" {
        return;
    }
    let exists =
        state.profiles.list().iter().any(|x| {
            x.source == "backup" && x.game == applied.game && x.source_account == account_id
        });
    let Some((name, icon)) = account_source(state, account_id).filter(|_| !exists) else {
        return;
    };
    let categories = if applied.game == "lol" {
        vec!["keybinds", "interface", "sound", "graphics"]
    } else {
        vec!["keybinds", "interface", "crosshair", "sound", "graphics"]
    };
    let p = ConfigProfile {
        name: name.split('#').next().unwrap_or(&name).to_string(),
        game: applied.game.clone(),
        source: "backup".into(),
        categories: categories.into_iter().map(String::from).collect(),
        source_account: account_id.to_string(),
        source_name: name,
        source_icon: icon,
        created_at: now_ms(),
        lol,
        val,
        ..Default::default()
    };
    let ok = state.profiles.save(p).is_ok();
    log(format!(
        "configs: sauvegarde automatique avant application ok={ok}"
    ));
}

/// Crée un profil depuis la config actuelle (client LoL connecté ou fichiers ; dossier Valorant choisi).
#[tauri::command]
pub async fn capture_profile(
    state: State<'_, AppState>,
    game: String,
    name: String,
    categories: Vec<String>,
    val_dir: Option<String>,
    source_account: Option<String>,
) -> AppResult<ConfigProfile> {
    state.touch();
    let settings = state.settings();
    let mut p = ConfigProfile {
        name: if name.trim().is_empty() {
            "Profil".into()
        } else {
            name.trim().to_string()
        },
        game: game.clone(),
        categories,
        created_at: now_ms(),
        ..Default::default()
    };
    if let Some(acc) = source_account {
        if let Some((name, icon)) = account_source(&state, &acc) {
            p.source_name = name;
            p.source_icon = icon;
        }
        p.source_account = acc;
    }
    match game.as_str() {
        "lol" => {
            let api = lcu::api(&settings.lol_path);
            let (cfg, source) = configs::capture_lol(api.as_ref(), &settings.lol_path).await?;
            p.lol = Some(cfg);
            p.source = source.into();
        }
        "val" => {
            let dir = val_dir.ok_or_else(|| AppError::code("val_dir_not_found"))?;
            p.val = Some(configs::capture_val(&dir)?);
            p.source = "files".into();
        }
        _ => return Err(AppError::new("invalid", "game")),
    }
    state.profiles.save(p)
}

/// Capture la config actuelle du client et renvoie seulement les touches (pour l'éditeur).
#[tauri::command]
pub async fn read_current_lol(state: State<'_, AppState>) -> AppResult<LolConfig> {
    let settings = state.settings();
    let api = lcu::api(&settings.lol_path);
    Ok(configs::capture_lol(api.as_ref(), &settings.lol_path)
        .await?
        .0)
}

#[tauri::command]
pub async fn apply_profile(
    state: State<'_, AppState>,
    id: String,
    val_dir: Option<String>,
) -> AppResult<Vec<String>> {
    state.touch();
    let settings = state.settings();
    let p = state.profiles.get(&id)?;
    match p.game.as_str() {
        "lol" => {
            let api = lcu::api(&settings.lol_path);
            let logged = match &api {
                Some(a) => lcu::logged_in(a).await,
                None => false,
            };
            let active = state.status.lock().unwrap().active_account_id.clone();
            if let (true, Some(acc), Some(a)) = (logged, active, api.as_ref()) {
                if let Ok((cfg, "client")) = configs::capture_lol(Some(a), &settings.lol_path).await
                {
                    backup_once(&state, &acc, &p, Some(cfg), None);
                }
            }
            configs::apply_lol(
                &p,
                if logged { api.as_ref() } else { None },
                &settings.lol_path,
            )
            .await
        }
        "val" => {
            let dir = val_dir.ok_or_else(|| AppError::code("val_dir_not_found"))?;
            let puuid = configs::list_val_dirs()
                .into_iter()
                .find(|d| d.name == dir)
                .map(|d| d.puuid);
            let owner = state
                .with_vault(|v| {
                    Ok(v.data
                        .accounts
                        .iter()
                        .find(|a| a.puuid.is_some() && a.puuid == puuid)
                        .map(|a| a.id.clone()))
                })
                .ok()
                .flatten();
            if let (Some(acc), Ok(cfg)) = (owner, configs::capture_val(&dir)) {
                backup_once(&state, &acc, &p, None, Some(cfg));
            }
            configs::apply_val(&p, &dir)
        }
        _ => Err(AppError::new("invalid", "game")),
    }
}

/// Associe un profil à plusieurs comptes d'un coup (ex. « nouvelle saison »).
#[tauri::command]
pub fn assign_profile(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    account_ids: Vec<String>,
) -> AppResult<()> {
    let p = state.profiles.get(&id)?;
    state.with_vault(|v| {
        for a in v
            .data
            .accounts
            .iter_mut()
            .filter(|a| account_ids.contains(&a.id))
        {
            match p.game.as_str() {
                "lol" => a.profiles.lol = Some(id.clone()),
                _ => a.profiles.val = Some(id.clone()),
            }
        }
        v.save()
    })?;
    changed(&app);
    Ok(())
}

#[tauri::command]
pub fn export_profile_code(state: State<'_, AppState>, id: String) -> AppResult<String> {
    configs::to_code(&state.profiles.get(&id)?)
}

#[tauri::command]
pub fn import_profile_code(state: State<'_, AppState>, code: String) -> AppResult<ConfigProfile> {
    state.profiles.save(configs::from_code(&code)?)
}

#[tauri::command]
pub fn export_profile_file(state: State<'_, AppState>, id: String, path: String) -> AppResult<()> {
    let mut p = state.profiles.get(&id)?;
    p.notes.clear();
    crate::util::atomic_write(
        std::path::Path::new(&path),
        &serde_json::to_vec_pretty(&p)?,
        false,
    )?;
    Ok(())
}

#[tauri::command]
pub fn import_profile_file(state: State<'_, AppState>, path: String) -> AppResult<ConfigProfile> {
    state
        .profiles
        .save(configs::read_file(std::path::Path::new(&path))?)
}

#[tauri::command]
pub fn list_val_dirs() -> Vec<ValDir> {
    configs::list_val_dirs()
}

// ── Runes ───────────────────────────────────────────────────────────────────

fn lcu_api(state: &AppState) -> AppResult<crate::riot::local_api::LocalApi> {
    lcu::api(&state.settings().lol_path).ok_or_else(|| AppError::code("lol_client_closed"))
}

#[tauri::command]
pub async fn runes_client_pages(state: State<'_, AppState>) -> AppResult<Vec<RunePage>> {
    let api = lcu_api(&state)?;
    runes::client_pages(&api).await
}

#[tauri::command]
pub fn runes_library(state: State<'_, AppState>) -> AppResult<Vec<RunePage>> {
    state.with_vault(|v| Ok(v.data.rune_library.clone()))
}

#[tauri::command]
pub fn runes_save(state: State<'_, AppState>, page: RunePage) -> AppResult<RunePage> {
    state.with_vault(|v| {
        let mut p = page;
        match v
            .data
            .rune_library
            .iter_mut()
            .find(|x| !p.id.is_empty() && x.id == p.id)
        {
            Some(existing) => *existing = p.clone(),
            None => {
                p.id = uuid::Uuid::new_v4().to_string();
                p.created_at = now_ms();
                v.data.rune_library.insert(0, p.clone());
            }
        }
        v.save()?;
        Ok(p)
    })
}

#[tauri::command]
pub fn runes_delete(state: State<'_, AppState>, id: String) -> AppResult<()> {
    state.with_vault(|v| {
        v.data.rune_library.retain(|p| p.id != id);
        v.save()
    })
}

#[tauri::command]
pub async fn runes_push(state: State<'_, AppState>, page: RunePage) -> AppResult<()> {
    let api = lcu_api(&state)?;
    runes::push_page(&api, &page).await
}

#[tauri::command]
pub fn runes_export_code(page: RunePage) -> String {
    runes::to_code(&page)
}

#[tauri::command]
pub fn runes_import_code(state: State<'_, AppState>, code: String) -> AppResult<RunePage> {
    let page = runes::from_code(&code)?;
    state.with_vault(|v| {
        v.data.rune_library.insert(0, page.clone());
        v.save()
    })?;
    Ok(page)
}

/// Lecture brute (lecture seule) d'une route LCU documentée pour l'affichage (ex. boutique, missions).
#[tauri::command]
pub async fn lcu_get(state: State<'_, AppState>, path: String) -> AppResult<Value> {
    const ALLOWED: [&str; 3] = [
        "/lol-store/v1/",
        "/lol-missions/v1/",
        "/lol-collections/v1/",
    ];
    if !ALLOWED.iter().any(|p| path.starts_with(p)) {
        return Err(AppError::new("invalid", "route"));
    }
    lcu_api(&state)?.get(&path).await
}
