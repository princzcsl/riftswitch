//! Préférences non sensibles, stockées en clair (`settings.json`).
//! Aucune donnée de compte ici : tout ce qui concerne les comptes vit dans le coffre chiffré.

use serde::{Deserialize, Serialize};

use crate::paths::Paths;
use crate::util;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    /// Assistant de premier lancement terminé.
    pub setup_done: bool,
    pub language: String,
    pub theme: String,
    /// "auto" | "quality" | "performance"
    pub performance: String,
    /// Zoom de l'interface en % (80–150), en plus de l'échelle Windows / macOS.
    pub zoom: u32,
    pub streamer_mode: bool,
    /// Affiche le libellé texte à côté des icônes de jeu dans le menu Jouer.
    pub play_menu_labels: bool,
    /// Verrouillage auto : 0 = désactivé (défaut).
    pub auto_lock_minutes: u32,
    pub lock_on_sleep: bool,
    /// Fermer la fenêtre = rester dans la zone de notification.
    pub close_to_tray: bool,
    pub start_minimized: bool,
    /// Fermer le Riot Client proprement avant un switch (sinon on refuse si une partie est en cours).
    pub block_switch_in_game: bool,
    /// Ferme la fenêtre du Riot Client (comme la croix) une fois le jeu ouvert.
    pub close_riot_client: bool,
    /// Appliquer automatiquement le profil de config associé au compte au lancement.
    pub auto_apply_profiles: bool,
    pub discord_rpc: bool,
    pub discord_show_riot_id: bool,
    pub discord_off_when_offline: bool,
    /// Fonctions à risque, opt-in.
    pub valorant_store_opt_in: bool,
    pub third_party_ranks_opt_in: bool,
    /// Télémétrie : opt-in, décochée par défaut. Rien n'est envoyé tant qu'elle est désactivée.
    pub telemetry: bool,
    pub telemetry_install_id: String,
    /// Jour (depuis l'époque Unix) du dernier résumé envoyé. Géré par le cœur uniquement.
    pub telemetry_last_day: String,
    /// Vérifie au démarrage si une nouvelle version est publiée (GitHub Releases).
    pub check_updates: bool,
    /// Éclats offerts en plus de ceux des succès (réglage manuel, absent de l'interface).
    pub bonus_shards: u32,
    /// Dernier jeu choisi globalement (pré-sélection).
    pub last_game: String,
    /// Chemins personnalisés (vides = détection automatique).
    pub riot_client_path: String,
    pub lol_path: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            setup_done: false,
            language: String::new(),
            theme: "liquid-glass-dark".into(),
            performance: "auto".into(),
            zoom: 100,
            streamer_mode: false,
            play_menu_labels: true,
            auto_lock_minutes: 0,
            lock_on_sleep: false,
            close_to_tray: true,
            start_minimized: false,
            block_switch_in_game: true,
            close_riot_client: true,
            auto_apply_profiles: true,
            discord_rpc: false,
            discord_show_riot_id: false,
            discord_off_when_offline: true,
            valorant_store_opt_in: false,
            third_party_ranks_opt_in: false,
            telemetry: false,
            telemetry_install_id: String::new(),
            telemetry_last_day: String::new(),
            check_updates: true,
            bonus_shards: 0,
            last_game: "lol".into(),
            riot_client_path: String::new(),
            lol_path: String::new(),
        }
    }
}

impl Settings {
    pub fn load(paths: &Paths) -> Self {
        std::fs::read(paths.settings())
            .ok()
            .and_then(|b| serde_json::from_slice::<Settings>(&b).ok())
            .map(|s| s.sanitized())
            .unwrap_or_default()
    }

    pub fn save(&self, paths: &Paths) -> std::io::Result<()> {
        let json = serde_json::to_vec_pretty(&self.clone().sanitized()).unwrap_or_default();
        util::atomic_write(&paths.settings(), &json, false)
    }

    pub fn sanitized(mut self) -> Self {
        self.zoom = self.zoom.clamp(80, 150);
        if !matches!(
            self.performance.as_str(),
            "auto" | "quality" | "performance"
        ) {
            self.performance = "auto".into();
        }
        if self.auto_lock_minutes > 24 * 60 {
            self.auto_lock_minutes = 24 * 60;
        }
        self
    }
}
