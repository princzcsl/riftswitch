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
mod schedule;
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
            play::play_external(app.clone(), id.clone(), game.clone(), false);
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
            appcmd::exit_prompt,
            appcmd::answer_exit,
            appcmd::take_tray_play,
            appcmd::install_update,
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
            // Fermeture de la mini-fenêtre de confirmation : ce n'est pas une demande de sortie.
            if code.is_none() && CLOSING_CONFIRM.swap(false, Ordering::Relaxed) {
                api.prevent_exit();
                return;
            }
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
static CLOSING_CONFIRM: AtomicBool = AtomicBool::new(false);

/// Petite fenêtre de confirmation, seule à l'écran (la fenêtre principale reste cachée).
fn open_confirm_window(app: &tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("confirm") {
        let _ = w.set_focus();
        return;
    }
    let _ = tauri::WebviewWindowBuilder::new(
        app,
        "confirm",
        tauri::WebviewUrl::App("index.html?view=confirm".into()),
    )
    .title("RiftSwitch")
    .inner_size(440.0, 220.0)
    .resizable(false)
    .decorations(false)
    .transparent(true)
    .shadow(false)
    .always_on_top(true)
    .center()
    .focused(true)
    .build();
}

fn close_confirm_window(app: &tauri::AppHandle) {
    if let Some(w) = app.get_webview_window("confirm") {
        CLOSING_CONFIRM.store(true, Ordering::Relaxed);
        let _ = w.destroy();
    }
}

/// Confirmation dans une mini-fenêtre au style de RiftSwitch.
async fn confirm_exit(app: tauri::AppHandle, ghost: bool, game_open: bool) {
    if game_open {
        let procs = riot::presence::with_match_state(riot::process::snapshot()).await;
        let lol_path = app.state::<AppState>().settings().lol_path;
        let in_select = commands::play::in_selection(&lol_path, &procs).await;
        let payload = serde_json::json!({
            "ghost": ghost,
            "inMatch": procs.game_in_progress(),
            "inSelect": in_select,
        });
        let (tx, rx) = tokio::sync::oneshot::channel();
        *app.state::<AppState>().pending_exit.lock().unwrap() = Some((payload, tx));
        open_confirm_window(&app);
        let ok = tokio::time::timeout(std::time::Duration::from_secs(300), rx)
            .await
            .ok()
            .and_then(Result::ok)
            .unwrap_or(false);
        app.state::<AppState>().pending_exit.lock().unwrap().take();
        close_confirm_window(&app);
        if !ok {
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
