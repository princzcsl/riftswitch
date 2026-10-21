//! Intégration Riot : uniquement des fichiers du Riot Client et des API locales prévues pour les
//! clients. Aucune lecture / écriture de la mémoire d'un jeu, aucune injection (compatible Vanguard).

pub mod captcha;
pub mod identity;
pub mod launch;
pub mod lcu;
pub mod local_api;
pub mod paths;
pub mod presence;
pub mod process;
pub mod redeem;
pub mod rso_login;
pub mod session;
pub mod window;

use serde::Serialize;

#[derive(Debug, Clone, Default, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RiotStatus {
    pub client_installed: bool,
    pub procs: process::ProcSnapshot,
    pub identity: Option<identity::ActiveIdentity>,
    pub active_account_id: Option<String>,
    pub proxy_running: bool,
    pub offline: bool,
    /// Statut affiché aux amis quand le client passe par le proxy : online, away, busy, mobile, offline.
    pub chat_status: Option<String>,
    /// Opération en cours : "switching" | "waiting_login" | None.
    pub busy: Option<String>,
}
