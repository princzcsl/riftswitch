//! Zone de notification : ouvrir, 3 comptes jouables en 2 clics, statut fantôme, verrouiller, quitter.

use std::sync::atomic::Ordering;
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, Wry};

use crate::state::AppState;

/// Libellé selon la langue : [en, fr, es, it, de].
fn pick(lang: &str, t: [&'static str; 5]) -> &'static str {
    match lang.get(..2).unwrap_or("en") {
        "fr" => t[1],
        "es" => t[2],
        "it" => t[3],
        "de" => t[4],
        _ => t[0],
    }
}

fn tr(lang: &str, key: &str) -> &'static str {
    let fr = lang.starts_with("fr");
    let es = lang.starts_with("es");
    let it = lang.starts_with("it");
    let de = lang.starts_with("de");
    match key {
        "open" => {
            if fr {
                "Ouvrir RiftSwitch"
            } else if es {
                "Abrir RiftSwitch"
            } else if it {
                "Apri RiftSwitch"
            } else if de {
                "RiftSwitch öffnen"
            } else {
                "Open RiftSwitch"
            }
        }
        "lock" => {
            if fr {
                "Verrouiller"
            } else if es {
                "Bloquear"
            } else if it {
                "Blocca"
            } else if de {
                "Sperren"
            } else {
                "Lock"
            }
        }
        "locked" => {
            if fr {
                "Coffre verrouillé"
            } else if es {
                "Bóveda bloqueada"
            } else if it {
                "Cassaforte bloccata"
            } else if de {
                "Tresor gesperrt"
            } else {
                "Vault locked"
            }
        }
        "quit" => {
            if fr {
                "Quitter"
            } else if es {
                "Salir"
            } else if it {
                "Esci"
            } else if de {
                "Beenden"
            } else {
                "Quit"
            }
        }
        "noAccounts" => pick(lang, ["No accounts", "Aucun compte", "Sin cuentas", "Nessun account", "Keine Konten"]),
        "status" => pick(lang, ["Status", "Statut", "Estado", "Stato", "Status"]),
        "online" => pick(lang, ["Online", "Connecté", "Conectado", "Online", "Online"]),
        "away" => pick(lang, ["Away", "Absent", "Ausente", "Assente", "Abwesend"]),
        "mobile" => pick(lang, ["Mobile", "Mobile", "Móvil", "Mobile", "Mobil"]),
        "offline" => pick(lang, ["Offline", "Hors ligne", "Desconectado", "Offline", "Offline"]),
        _ => "",
    }
}

fn build_menu(app: &AppHandle) -> tauri::Result<Menu<Wry>> {
    let state = app.state::<AppState>();
    let settings = state.settings();
    let lang = settings.language.clone();
    let menu = Menu::new(app)?;
    menu.append(&MenuItem::with_id(
        app,
        "open",
        tr(&lang, "open"),
        true,
        None::<&str>,
    )?)?;
    menu.append(&PredefinedMenuItem::separator(app)?)?;

    // Statut affiché aux amis, quand le jeu passe par le mode fantôme.
    if let Some(current) = state.status.lock().unwrap().chat_status.clone() {
        let items = ["online", "away", "mobile", "offline"]
            .iter()
            .map(|s| CheckMenuItem::with_id(app, format!("status:{s}"), tr(&lang, s), true, current == *s, None::<&str>))
            .collect::<tauri::Result<Vec<_>>>()?;
        let refs: Vec<&dyn tauri::menu::IsMenuItem<Wry>> = items.iter().map(|i| i as &dyn tauri::menu::IsMenuItem<Wry>).collect();
        menu.append(&Submenu::with_items(app, tr(&lang, "status"), true, &refs)?)?;
        menu.append(&PredefinedMenuItem::separator(app)?)?;
    }

    // Jusqu'à 3 comptes : favoris d'abord, puis les plus récemment utilisés.
    let active = state.status.lock().unwrap().active_account_id.clone();
    let accounts = state
        .with_vault(|v| {
            let mut list: Vec<_> = v.data.accounts.iter().collect();
            list.sort_by_key(|a| (!a.favorite, std::cmp::Reverse(a.last_used_at.unwrap_or(0))));
            Ok(list
                .into_iter()
                .take(3)
                .map(|a| {
                    let base = if a.label.is_empty() { a.game_name.clone() } else { a.label.clone() };
                    let name = if settings.streamer_mode || a.riot_id().is_empty() || a.label.is_empty() {
                        base
                    } else {
                        format!("{base} · {}", a.riot_id())
                    };
                    let name = if active.as_deref() == Some(a.id.as_str()) { format!("✓ {name}") } else { name };
                    (a.id.clone(), name)
                })
                .collect::<Vec<_>>())
        })
        .ok();

    match accounts {
        None => {
            menu.append(&MenuItem::with_id(app, "noop", tr(&lang, "locked"), false, None::<&str>)?)?;
        }
        Some(list) if list.is_empty() => {
            menu.append(&MenuItem::with_id(app, "noop", tr(&lang, "noAccounts"), false, None::<&str>)?)?;
        }
        Some(list) => {
            for (id, name) in list {
                let sub = Submenu::with_items(
                    app,
                    name,
                    true,
                    &[
                        &MenuItem::with_id(app, format!("play:{id}:lol"), "League of Legends", true, None::<&str>)?,
                        &MenuItem::with_id(app, format!("play:{id}:tft"), "Teamfight Tactics", true, None::<&str>)?,
                        &MenuItem::with_id(app, format!("play:{id}:val"), "VALORANT", true, None::<&str>)?,
                    ],
                )?;
                menu.append(&sub)?;
            }
        }
    }

    menu.append(&PredefinedMenuItem::separator(app)?)?;
    let lockable = state
        .with_vault(|v| Ok(v.mode() == "password"))
        .unwrap_or(false);
    if lockable {
        menu.append(&MenuItem::with_id(
            app,
            "lock",
            tr(&lang, "lock"),
            true,
            None::<&str>,
        )?)?;
    }
    menu.append(&MenuItem::with_id(
        app,
        "quit",
        tr(&lang, "quit"),
        true,
        None::<&str>,
    )?)?;
    Ok(menu)
}

pub fn init(app: &AppHandle) -> tauri::Result<()> {
    let menu = build_menu(app)?;
    let mut builder = TrayIconBuilder::with_id("main")
        .tooltip("RiftSwitch")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| handle(app, event.id.as_ref()))
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;
    Ok(())
}

pub fn refresh(app: &AppHandle) {
    if let Some(tray) = app.tray_by_id("main") {
        if let Ok(menu) = build_menu(app) {
            let _ = tray.set_menu(Some(menu));
        }
    }
}

fn handle(app: &AppHandle, id: &str) {
    match id {
        "open" => show_main(app),
        "quit" => app.exit(0),
        "lock" => {
            let state = app.state::<AppState>();
            crate::commands::app::do_lock(app, &state);
        }
        other if other.starts_with("play:") => {
            // Exécuté par l'interface : préférence fantôme, confirmations et étapes affichées.
            let parts: Vec<&str> = other.splitn(3, ':').collect();
            if parts.len() == 3 {
                let state = app.state::<AppState>();
                *state.pending_tray_play.lock().unwrap() = Some((parts[1].to_string(), parts[2].to_string()));
                show_main(app);
                let _ = app.emit("tray-play", ());
            }
        }
        other if other.starts_with("status:") => {
            let state = app.state::<AppState>();
            if let (Some(p), Some(st)) = (state.proxy(), crate::offline::ChatStatus::parse(&other[7..])) {
                p.set_status(st);
                if let Some(cs) = state.status.lock().unwrap().chat_status.as_mut() {
                    *cs = st.as_str().to_string();
                }
                let _ = app.emit("riot-status", state.status.lock().unwrap().clone());
                refresh(app);
            }
        }
        _ => {}
    }
}

/// Affiche la fenêtre principale, en la recréant si elle a été détruite (mode zone de notification).
pub fn show_main(app: &AppHandle) {
    let state = app.state::<AppState>();
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    } else if let Some(cfg) = app
        .config()
        .app
        .windows
        .iter()
        .find(|w| w.label == "main")
        .cloned()
    {
        if let Ok(builder) = tauri::WebviewWindowBuilder::from_config(app, &cfg) {
            let _ = builder.build();
        }
    }
    state.window_visible.store(true, Ordering::Relaxed);
}
