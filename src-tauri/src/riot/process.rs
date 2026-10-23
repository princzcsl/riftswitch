//! Détection et arrêt des processus Riot. On ne lit ni n'écrit JAMAIS la mémoire d'un processus :
//! seulement la liste des noms (comme le Gestionnaire des tâches).

use serde::Serialize;
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};

const RIOT_CLIENT: &[&str] = &[
    "riotclientservices",
    "riot client",
    "riotclientux",
    "riotclientuxrender",
    "riotclientcrashhandler",
];
const LOL_CLIENT: &[&str] = &["leagueclient", "leagueclientux", "leagueclientuxrender"];
const LOL_GAME: &[&str] = &["league of legends"];
const VAL_GAME: &[&str] = &["valorant-win64-shipping", "valorant"];

#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProcSnapshot {
    pub riot_client: bool,
    pub riot_client_pid: Option<u32>,
    pub lol_client: bool,
    pub lol_game: bool,
    /// VALORANT ouvert (le processus du jeu tourne dès le menu principal).
    pub val_game: bool,
    /// Réellement en partie VALORANT : complété par `presence::with_match_state`.
    pub val_in_match: bool,
}

impl ProcSnapshot {
    /// Le produit demandé est ouvert : client LoL (LoL / TFT) ou jeu VALORANT.
    pub fn product_running(&self, game: &str) -> bool {
        if game == "val" {
            self.val_game
        } else {
            self.lol_client || self.lol_game
        }
    }
    pub fn game_in_progress(&self) -> bool {
        self.lol_game || self.val_in_match
    }
    pub fn any_riot(&self) -> bool {
        self.riot_client || self.lol_client || self.lol_game || self.val_game
    }
}

fn normalize(name: &std::ffi::OsStr) -> String {
    let n = name.to_string_lossy().to_lowercase();
    n.strip_suffix(".exe").map(str::to_string).unwrap_or(n)
}

fn refreshed() -> System {
    let mut sys = System::new();
    sys.refresh_processes_specifics(ProcessesToUpdate::All, true, ProcessRefreshKind::nothing());
    sys
}

/// PID des processus d'interface du Riot Client (ceux qui portent sa fenêtre).
pub fn riot_client_ui_pids() -> Vec<u32> {
    refreshed()
        .processes()
        .iter()
        .filter(|(_, p)| {
            let n = normalize(p.name());
            RIOT_CLIENT.contains(&n.as_str()) && n != "riotclientcrashhandler"
        })
        .map(|(pid, _)| pid.as_u32())
        .collect()
}

pub fn snapshot() -> ProcSnapshot {
    let sys = refreshed();
    let mut s = ProcSnapshot::default();
    for (pid, p) in sys.processes() {
        let n = normalize(p.name());
        if RIOT_CLIENT.contains(&n.as_str()) {
            s.riot_client = true;
            if n == "riotclientservices" {
                s.riot_client_pid = Some(pid.as_u32());
            }
        } else if LOL_CLIENT.contains(&n.as_str()) {
            s.lol_client = true;
        } else if LOL_GAME.contains(&n.as_str()) {
            s.lol_game = true;
        } else if VAL_GAME.contains(&n.as_str()) {
            s.val_game = true;
        }
    }
    s
}

/// Jeu auquel appartient un processus : "lol" (client ou partie) ou "val".
fn product_of(n: &str) -> Option<&'static str> {
    if LOL_CLIENT.contains(&n) || LOL_GAME.contains(&n) {
        Some("lol")
    } else if VAL_GAME.contains(&n) {
        Some("val")
    } else {
        None
    }
}

/// Ferme les processus d'un jeu ("lol" ou "val"), sans toucher au Riot Client.
pub fn kill_product(product: &str) -> usize {
    refreshed()
        .processes()
        .values()
        .filter(|p| product_of(&normalize(p.name())) == Some(product) && p.kill())
        .count()
}

/// LoL et VALORANT ouverts en même temps : renvoie le dernier lancé.
pub fn newest_product() -> Option<&'static str> {
    let (mut lol, mut val) = (0u64, 0u64);
    for p in refreshed().processes().values() {
        match product_of(&normalize(p.name())) {
            Some("lol") => lol = lol.max(p.start_time()),
            Some("val") => val = val.max(p.start_time()),
            _ => {}
        }
    }
    (lol > 0 && val > 0).then_some(if val >= lol { "val" } else { "lol" })
}

/// Tue tous les processus Riot restants (après une demande d'arrêt propre).
pub fn kill_all() -> usize {
    let sys = refreshed();
    let mut killed = 0;
    for p in sys.processes().values() {
        let n = normalize(p.name());
        let all = [RIOT_CLIENT, LOL_CLIENT, LOL_GAME, VAL_GAME];
        if all.iter().any(|set| set.contains(&n.as_str())) && p.kill() {
            killed += 1;
        }
    }
    killed
}
