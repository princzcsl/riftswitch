//! RiftSwitch — cœur Rust. Le front n'a jamais accès aux secrets : il demande « jouer sur le
//! compte X » et le cœur déchiffre, utilise et efface.

mod commands;
mod configs;
mod error;
mod monitor;
mod offline;
mod paths;
mod riot;
mod runes;
mod settings;
mod state;
mod telemetry;
mod tray;
mod util;
mod vault;

use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{Manager, RunEvent, WindowEvent};

use commands::{accounts, app as appcmd, configs as cfgcmd, play};
use state::AppState;

/// `--play <id> <jeu>` : lancement direct (raccourcis, Jump List).
fn handle_args(app: &tauri::AppHandle, argv: &[String]) -> bool {
    if let Some(pos) = argv.iter().position(|a| a == "--play") {
        if let (Some(id), Some(game)) = (argv.get(pos + 1), argv.get(pos + 2)) {
            play::play_external(app.clone(), id.clone(), game.clone());
            return true;
        }
    }
    false
}

pub fn run() {
    let paths = paths::Paths::resolve();
    let _ = std::fs::create_dir_all(&paths.root);
    util::init_log(paths.log());
    util::log(format!(
        "app: démarrage v{} portable={}",
        env!("CARGO_PKG_VERSION"),
        paths.portable
    ));

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            if !handle_args(app, &argv) {
                tray::show_main(app);
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .manage(AppState::new(paths))
        .setup(|app| {
            let handle = app.handle().clone();
            tray::init(&handle)?;
            monitor::spawn(handle.clone());
            let argv: Vec<String> = std::env::args().collect();
            handle_args(&handle, &argv);
            let state = app.state::<AppState>();
            let minimized =
                argv.iter().any(|a| a == "--minimized") || state.settings().start_minimized;
            if minimized && state.settings().setup_done {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.destroy();
                }
                state.window_visible.store(false, Ordering::Relaxed);
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::Destroyed = event {
                if window.label() == "main" {
                    window
                        .app_handle()
                        .state::<AppState>()
                        .window_visible
                        .store(false, Ordering::Relaxed);
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            appcmd::get_bootstrap,
            appcmd::save_settings,
            appcmd::touch,
            appcmd::setup_vault,
            appcmd::unlock,
            appcmd::lock,
            appcmd::change_master_password,
            appcmd::reset_app,
            appcmd::export_backup,
            appcmd::import_backup,
            appcmd::telemetry_preview,
            appcmd::close_window,
            appcmd::quit_app,
            appcmd::open_data_dir,
            accounts::list_accounts,
            accounts::save_account,
            accounts::set_favorite,
            accounts::delete_account,
            accounts::reorder_accounts,
            accounts::list_folders,
            accounts::save_folder,
            accounts::delete_folder,
            accounts::reorder_folders,
            play::play,
            play::set_offline,
            play::set_chat_status,
            play::play_check,
            play::riot_status,
            play::quit_riot,
            play::capture_current,
            play::add_via_riot_login,
            play::cancel_login,
            play::submit_mfa,
            cfgcmd::list_profiles,
            cfgcmd::get_profile,
            cfgcmd::save_profile,
            cfgcmd::delete_profile,
            cfgcmd::duplicate_profile,
            cfgcmd::capture_profile,
            cfgcmd::read_current_lol,
            cfgcmd::apply_profile,
            cfgcmd::assign_profile,
            cfgcmd::export_profile_code,
            cfgcmd::import_profile_code,
            cfgcmd::export_profile_file,
            cfgcmd::import_profile_file,
            cfgcmd::list_val_dirs,
            cfgcmd::runes_client_pages,
            cfgcmd::runes_library,
            cfgcmd::runes_save,
            cfgcmd::runes_delete,
            cfgcmd::runes_push,
            cfgcmd::runes_export_code,
            cfgcmd::runes_import_code,
            cfgcmd::lcu_get,
        ])
        .build(tauri::generate_context!())
        .expect("impossible de démarrer RiftSwitch");

    app.run(|app, event| {
        if let RunEvent::ExitRequested { api, code, .. } = event {
            let state = app.state::<AppState>();
            // Fenêtre fermée : on reste dans la zone de notification si l'option est active.
            if code.is_none() && state.settings().close_to_tray {
                api.prevent_exit();
                return;
            }
            if EXITING.load(Ordering::Relaxed) {
                return;
            }
            // Mode fantôme : sans le proxy, le chat du jeu tombe, donc le jeu est fermé avec l'app.
            // Jeu ouvert sans mode fantôme : simple confirmation, le jeu reste ouvert.
            let ghost = state.client_via_proxy.load(Ordering::Relaxed) && state.proxy().is_some();
            let procs = riot::process::snapshot();
            let game_open = procs.lol_client || procs.lol_game || procs.val_game;
            if ghost || game_open {
                api.prevent_exit();
                EXITING.store(true, Ordering::Relaxed);
                tauri::async_runtime::spawn(confirm_exit(app.clone(), ghost, game_open));
            }
        }
    });
}

static EXITING: AtomicBool = AtomicBool::new(false);

/// Textes de la confirmation de fermeture : (titre, texte, bouton de validation, annuler).
fn exit_texts(lang: &str, ghost: bool, in_match: bool) -> (String, String, String, String) {
    let l = lang.get(..2).unwrap_or("en");
    let (title, ghost_text, open_text, quit_close, quit, cancel, abandon) = match l {
        "fr" => ("Fermer RiftSwitch", "Le mode fantôme est actif : quitter RiftSwitch fermera aussi le jeu.", "Un jeu est ouvert. Quitter RiftSwitch quand même ? Le jeu reste ouvert.", "Quitter et fermer le jeu", "Quitter", "Annuler", "La partie en cours sera abandonnée."),
        "es" => ("Cerrar RiftSwitch", "El modo fantasma está activo: salir de RiftSwitch también cerrará el juego.", "Hay un juego abierto. ¿Salir de RiftSwitch de todos modos? El juego seguirá abierto.", "Salir y cerrar el juego", "Salir", "Cancelar", "La partida en curso se abandonará."),
        "it" => ("Chiudi RiftSwitch", "La modalità fantasma è attiva: uscire da RiftSwitch chiuderà anche il gioco.", "Un gioco è aperto. Uscire comunque da RiftSwitch? Il gioco resterà aperto.", "Esci e chiudi il gioco", "Esci", "Annulla", "La partita in corso verrà abbandonata."),
        "de" => ("RiftSwitch beenden", "Der Geistermodus ist aktiv: Beim Beenden von RiftSwitch wird auch das Spiel geschlossen.", "Ein Spiel ist geöffnet. RiftSwitch trotzdem beenden? Das Spiel bleibt geöffnet.", "Beenden und Spiel schließen", "Beenden", "Abbrechen", "Das laufende Match wird abgebrochen."),
        _ => ("Quit RiftSwitch", "Ghost mode is on: quitting RiftSwitch also closes the game.", "A game is open. Quit RiftSwitch anyway? The game stays open.", "Quit and close the game", "Quit", "Cancel", "The current match will be abandoned."),
    };
    let text = if ghost && in_match {
        format!("{ghost_text} {abandon}")
    } else if ghost {
        ghost_text.to_string()
    } else {
        open_text.to_string()
    };
    let ok = if ghost { quit_close } else { quit };
    (title.into(), text, ok.into(), cancel.into())
}

async fn confirm_exit(app: tauri::AppHandle, ghost: bool, game_open: bool) {
    use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
    if game_open {
        let in_match = riot::presence::with_match_state(riot::process::snapshot())
            .await
            .game_in_progress();
        let lang = app.state::<AppState>().settings().language;
        let (title, text, yes, no) = exit_texts(&lang, ghost, in_match);
        let (tx, rx) = tokio::sync::oneshot::channel();
        app.dialog()
            .message(text)
            .title(title)
            .kind(MessageDialogKind::Warning)
            .buttons(MessageDialogButtons::OkCancelCustom(yes, no))
            .show(move |ok| {
                let _ = tx.send(ok);
            });
        if !rx.await.unwrap_or(false) {
            EXITING.store(false, Ordering::Relaxed);
            return;
        }
    }
    if ghost {
        util::log("app: fermeture en mode fantôme, fermeture du jeu");
        riot::launch::shutdown(std::time::Duration::from_secs(8)).await;
    }
    app.exit(0);
}
