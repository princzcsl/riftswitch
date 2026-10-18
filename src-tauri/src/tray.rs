//! Zone de notification : ouvrir, favoris jouables en 2 clics, verrouiller, quitter.

use std::sync::atomic::Ordering;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Wry};

use crate::state::AppState;

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
        "favorites" => {
            if fr {
                "Aucun favori"
            } else if es {
                "Sin favoritos"
            } else if it {
                "Nessun preferito"
            } else if de {
                "Keine Favoriten"
            } else {
                "No favorites"
            }
        }
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

    let favorites = state
        .with_vault(|v| {
            Ok(v.data
                .accounts
                .iter()
                .filter(|a| a.favorite)
                .take(8)
                .map(|a| {
                    let name = if settings.streamer_mode || a.riot_id().is_empty() {
                        a.label.clone()
                    } else {
                        format!("{} · {}", a.label, a.riot_id())
                    };
                    (a.id.clone(), name)
                })
                .collect::<Vec<_>>())
        })
        .ok();

    match favorites {
        None => {
            menu.append(&MenuItem::with_id(
                app,
                "noop",
                tr(&lang, "locked"),
                false,
                None::<&str>,
            )?)?;
        }
        Some(list) if list.is_empty() => {
            menu.append(&MenuItem::with_id(
                app,
                "noop",
                tr(&lang, "favorites"),
                false,
                None::<&str>,
            )?)?;
        }
        Some(list) => {
            for (id, name) in list {
                let sub = Submenu::with_items(
                    app,
                    name,
                    true,
                    &[
                        &MenuItem::with_id(
                            app,
                            format!("play:{id}:lol"),
                            "League of Legends",
                            true,
                            None::<&str>,
                        )?,
                        &MenuItem::with_id(
                            app,
                            format!("play:{id}:tft"),
                            "Teamfight Tactics",
                            true,
                            None::<&str>,
                        )?,
                        &MenuItem::with_id(
                            app,
                            format!("play:{id}:val"),
                            "VALORANT",
                            true,
                            None::<&str>,
                        )?,
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
            let parts: Vec<&str> = other.splitn(3, ':').collect();
            if parts.len() == 3 {
                crate::commands::play::play_external(
                    app.clone(),
                    parts[1].to_string(),
                    parts[2].to_string(),
                );
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
