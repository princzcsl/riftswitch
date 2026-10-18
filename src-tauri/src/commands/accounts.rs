//! Comptes et dossiers. Le front reçoit des vues sans secret (email masqué, indicateurs booléens).

use serde::Deserialize;
use serde_json::Value;
use tauri::{AppHandle, Emitter, State};

use crate::error::{AppError, AppResult};
use crate::state::AppState;
use crate::util::{mask_email, now_ms};
use crate::vault::model::{Account, Folder, GameLangs, ProfileLinks, Ranks};

const PALETTE: [&str; 10] = [
    "#7c5cff", "#2fd4ff", "#ff5c8a", "#ffb547", "#3ddc97", "#ff7a45", "#5c8dff", "#c86bff",
    "#00c2a8", "#e8c547",
];

pub fn account_view(a: &Account) -> Value {
    let mut v = serde_json::to_value(a).unwrap_or(Value::Null);
    if let Some(obj) = v.as_object_mut() {
        obj.remove("secret");
        obj.remove("email");
        obj.insert(
            "maskedEmail".into(),
            Value::String(if a.email.is_empty() {
                String::new()
            } else {
                mask_email(&a.email)
            }),
        );
        obj.insert("hasSession".into(), Value::Bool(a.has_tokens));
        obj.insert("riotId".into(), Value::String(a.riot_id()));
    }
    v
}

pub fn new_account(state: &AppState) -> Account {
    let n = state.with_vault(|v| Ok(v.data.accounts.len())).unwrap_or(0);
    Account {
        id: uuid::Uuid::new_v4().to_string(),
        color: PALETTE[n % PALETTE.len()].to_string(),
        icon: format!("glyph:{}", n % 12),
        created_at: now_ms(),
        ..Default::default()
    }
}

pub fn changed(app: &AppHandle) {
    let _ = app.emit("accounts-changed", ());
    crate::tray::refresh(app);
}

#[tauri::command]
pub fn list_accounts(state: State<'_, AppState>) -> AppResult<Vec<Value>> {
    state.touch();
    state.with_vault(|v| Ok(v.data.accounts.iter().map(account_view).collect()))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountInput {
    pub id: Option<String>,
    pub label: String,
    pub game_name: String,
    pub tag_line: String,
    pub region: String,
    pub color: String,
    pub icon: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub folder_id: Option<String>,
    #[serde(default)]
    pub favorite: bool,
    #[serde(default)]
    pub langs: GameLangs,
    #[serde(default)]
    pub two_fa: bool,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub champion_pool: Vec<String>,
    #[serde(default)]
    pub agent_pool: Vec<String>,
    #[serde(default)]
    pub roles: Vec<String>,
    #[serde(default)]
    pub profiles: ProfileLinks,
    /// Rangs saisis à la main (Valorant notamment) ; None = inchangés.
    pub ranks: Option<Ranks>,
    /// None = inchangé ; Some("") = effacé. Le front ne relit jamais ces valeurs.
    pub email: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[tauri::command]
pub fn save_account(
    app: AppHandle,
    state: State<'_, AppState>,
    input: AccountInput,
) -> AppResult<Value> {
    state.touch();
    if input.icon.len() > 300_000 {
        return Err(AppError::code("icon_too_large"));
    }
    let fresh = new_account(&state);
    let view = state.with_vault(|v| {
        let idx = match input.id.as_deref().filter(|s| !s.is_empty()) {
            Some(id) => v.index_of(id)?,
            None => {
                v.data.accounts.push(fresh);
                v.data.accounts.len() - 1
            }
        };
        let now = now_ms();
        {
            let a = &mut v.data.accounts[idx];
            a.label = input.label.trim().chars().take(40).collect();
            a.observe_riot_id(
                input.game_name.trim(),
                input.tag_line.trim().trim_start_matches('#'),
                now,
            );
            if input.game_name.trim().is_empty() {
                a.game_name.clear();
                a.tag_line.clear();
            }
            a.region = input.region;
            a.color = input.color;
            a.icon = input.icon;
            a.tags = input
                .tags
                .into_iter()
                .map(|t| t.trim().to_string())
                .filter(|t| !t.is_empty())
                .take(12)
                .collect();
            a.folder_id = input.folder_id.filter(|f| !f.is_empty());
            a.favorite = input.favorite;
            a.langs = input.langs;
            a.two_fa = input.two_fa;
            a.notes = input.notes;
            a.champion_pool = input.champion_pool;
            a.agent_pool = input.agent_pool;
            a.roles = input.roles;
            a.profiles = input.profiles;
            if let Some(mut r) = input.ranks {
                r.manual = true;
                r.updated_at = Some(now);
                a.ranks = r;
            }
            if let Some(e) = input.email {
                a.email = e.trim().to_string();
            }
        }
        if input.username.is_some() || input.password.is_some() {
            let (user, pass) = (input.username.clone(), input.password.clone());
            let id = v.data.accounts[idx].id.clone();
            v.update_secrets(&id, |s| {
                if let Some(u) = user {
                    s.username = if u.is_empty() { None } else { Some(u) };
                }
                if let Some(p) = pass {
                    s.password = if p.is_empty() { None } else { Some(p) };
                }
            })?;
            let secrets = v.open_secrets(&v.data.accounts[idx])?;
            let a = &mut v.data.accounts[idx];
            a.has_username = secrets.username.is_some();
            a.has_password = secrets.password.is_some();
        }
        v.save()?;
        Ok(account_view(&v.data.accounts[idx]))
    })?;
    changed(&app);
    Ok(view)
}

#[tauri::command]
pub fn set_favorite(
    app: AppHandle,
    state: State<'_, AppState>,
    id: String,
    favorite: bool,
) -> AppResult<()> {
    state.with_vault(|v| {
        let idx = v.index_of(&id)?;
        v.data.accounts[idx].favorite = favorite;
        v.save()
    })?;
    changed(&app);
    Ok(())
}

#[tauri::command]
pub fn delete_account(app: AppHandle, state: State<'_, AppState>, id: String) -> AppResult<()> {
    state.touch();
    state.with_vault(|v| {
        let idx = v.index_of(&id)?;
        v.data.accounts.remove(idx);
        if v.data.last_active_account.as_deref() == Some(id.as_str()) {
            v.data.last_active_account = None;
        }
        v.save()
    })?;
    changed(&app);
    Ok(())
}

#[tauri::command]
pub fn reorder_accounts(
    app: AppHandle,
    state: State<'_, AppState>,
    ids: Vec<String>,
) -> AppResult<()> {
    state.with_vault(|v| {
        v.data
            .accounts
            .sort_by_key(|a| ids.iter().position(|i| i == &a.id).unwrap_or(usize::MAX));
        v.save()
    })?;
    changed(&app);
    Ok(())
}

// ── Dossiers ────────────────────────────────────────────────────────────────

#[tauri::command]
pub fn list_folders(state: State<'_, AppState>) -> AppResult<Vec<Folder>> {
    state.with_vault(|v| {
        let mut f = v.data.folders.clone();
        f.sort_by_key(|x| x.order);
        Ok(f)
    })
}

#[tauri::command]
pub fn save_folder(
    app: AppHandle,
    state: State<'_, AppState>,
    folder: Folder,
) -> AppResult<Folder> {
    let saved = state.with_vault(|v| {
        let mut f = folder;
        f.name = f.name.trim().chars().take(30).collect();
        if f.name.is_empty() {
            return Err(AppError::code("name_required"));
        }
        match v
            .data
            .folders
            .iter_mut()
            .find(|x| !f.id.is_empty() && x.id == f.id)
        {
            Some(existing) => *existing = f.clone(),
            None => {
                f.id = uuid::Uuid::new_v4().to_string();
                f.order = v.data.folders.len() as i32;
                v.data.folders.push(f.clone());
            }
        }
        v.save()?;
        Ok(f)
    })?;
    changed(&app);
    Ok(saved)
}

#[tauri::command]
pub fn delete_folder(app: AppHandle, state: State<'_, AppState>, id: String) -> AppResult<()> {
    state.with_vault(|v| {
        v.data.folders.retain(|f| f.id != id);
        for a in v.data.accounts.iter_mut() {
            if a.folder_id.as_deref() == Some(id.as_str()) {
                a.folder_id = None;
            }
        }
        v.save()
    })?;
    changed(&app);
    Ok(())
}

#[tauri::command]
pub fn reorder_folders(
    app: AppHandle,
    state: State<'_, AppState>,
    ids: Vec<String>,
) -> AppResult<()> {
    state.with_vault(|v| {
        for f in v.data.folders.iter_mut() {
            f.order = ids
                .iter()
                .position(|i| i == &f.id)
                .map(|p| p as i32)
                .unwrap_or(i32::MAX);
        }
        v.save()
    })?;
    changed(&app);
    Ok(())
}
