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
    pub val_game: bool,
}

impl ProcSnapshot {
    pub fn game_in_progress(&self) -> bool {
        self.lol_game || self.val_game
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
