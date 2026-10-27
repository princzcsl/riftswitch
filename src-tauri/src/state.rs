//! État global de l'application, partagé entre commandes, surveillance de fond et zone de notification.
//! Règle : ne jamais garder un verrou `std::sync::Mutex` pendant un `.await`.

use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU32};
use std::sync::{Arc, Mutex};

use crate::configs::ProfileStore;
use crate::error::{AppError, AppResult};
use crate::offline::OfflineProxy;
use crate::paths::Paths;
use crate::riot::RiotStatus;
use crate::settings::Settings;
use crate::util::now_ms;
use crate::vault::Vault;

pub struct AppState {
    pub paths: Paths,
    pub settings: Mutex<Settings>,
    pub vault: Mutex<Option<Vault>>,
    pub profiles: ProfileStore,
    pub status: Mutex<RiotStatus>,
    pub offline: Mutex<Option<Arc<OfflineProxy>>>,
    pub last_activity: AtomicI64,
    pub failed_unlocks: AtomicU32,
    /// Temps de jeu crédité pendant que le coffre était verrouillé : (compte, jeu, secondes).
    pub pending_playtime: Mutex<Vec<(String, String, u64)>>,
    /// Une seule bascule / connexion à la fois.
    pub busy: tokio::sync::Mutex<()>,
    pub cancel_login: AtomicBool,
    pub window_visible: AtomicBool,
    /// Force une synchro LCU au prochain tour de surveillance.
    pub force_sync: AtomicBool,
    /// Le Riot Client en cours a été lancé à travers le proxy hors ligne.
    pub client_via_proxy: AtomicBool,
    /// Mode fantôme du lancement en cours, modifiable tant que le Riot Client n'est pas démarré.
    pub launch_ghost: AtomicBool,
    pub launch_ghost_open: AtomicBool,
    /// Émetteur du code A2F attendu pendant une connexion (rempli par la commande `submit_mfa`).
    pub pending_otp: tokio::sync::Mutex<Option<tokio::sync::oneshot::Sender<String>>>,
    /// Confirmation de fermeture affichée par l'interface : (contenu, réponse attendue).
    pub pending_exit: Mutex<Option<(serde_json::Value, tokio::sync::oneshot::Sender<bool>)>>,
    /// Lancement demandé depuis la zone de notification, exécuté par l'interface : (compte, jeu).
    pub pending_tray_play: Mutex<Option<(String, String)>>,
}

impl AppState {
    pub fn new(paths: Paths) -> Self {
        let settings = Settings::load(&paths);
        let profiles = ProfileStore::new(paths.profiles_dir());
        Self {
            settings: Mutex::new(settings),
            vault: Mutex::new(None),
            profiles,
            status: Mutex::new(RiotStatus::default()),
            offline: Mutex::new(None),
            last_activity: AtomicI64::new(now_ms()),
            failed_unlocks: AtomicU32::new(0),
            pending_playtime: Mutex::new(Vec::new()),
            busy: tokio::sync::Mutex::new(()),
            cancel_login: AtomicBool::new(false),
            window_visible: AtomicBool::new(true),
            force_sync: AtomicBool::new(true),
            client_via_proxy: AtomicBool::new(false),
            launch_ghost: AtomicBool::new(false),
            launch_ghost_open: AtomicBool::new(false),
            pending_otp: tokio::sync::Mutex::new(None),
            pending_exit: Mutex::new(None),
            pending_tray_play: Mutex::new(None),
            paths,
        }
    }

    pub fn settings(&self) -> Settings {
        self.settings.lock().unwrap().clone()
    }

    pub fn touch(&self) {
        self.last_activity
            .store(now_ms(), std::sync::atomic::Ordering::Relaxed);
    }

    /// Exécute `f` sur le coffre déverrouillé.
    pub fn with_vault<T>(&self, f: impl FnOnce(&mut Vault) -> AppResult<T>) -> AppResult<T> {
        let mut guard = self.vault.lock().unwrap();
        let vault = guard
            .as_mut()
            .ok_or_else(|| AppError::code("vault_locked"))?;
        f(vault)
    }

    pub fn is_unlocked(&self) -> bool {
        self.vault.lock().unwrap().is_some()
    }

    pub fn proxy(&self) -> Option<Arc<OfflineProxy>> {
        self.offline.lock().unwrap().clone()
    }
}
