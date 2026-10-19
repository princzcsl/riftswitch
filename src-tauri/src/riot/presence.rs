//! État de partie VALORANT lu dans la présence de chat locale du Riot Client (lecture seule).
//! Le processus du jeu tourne dès le menu principal : seul l'état de présence dit si l'on est
//! réellement en partie (`MENUS`, `PREGAME`, `INGAME`).

use base64::Engine;
use serde_json::Value;

use super::identity;
use super::process::ProcSnapshot;

/// Extrait l'état de boucle de session du champ `private` (JSON encodé en base64).
/// Gère l'ancien format (`sessionLoopState` à la racine) et le nouveau (`matchPresenceData`).
pub fn loop_state(private_b64: &str) -> Option<String> {
    let raw = base64::engine::general_purpose::STANDARD
        .decode(private_b64.trim())
        .ok()?;
    let v: Value = serde_json::from_slice(&raw).ok()?;
    v.get("sessionLoopState")
        .or_else(|| v.get("matchPresenceData")?.get("sessionLoopState"))
        .and_then(|s| s.as_str())
        .map(str::to_ascii_uppercase)
}

/// Le compte connecté est-il en partie VALORANT (`INGAME`) ?
pub async fn valorant_in_match() -> bool {
    let Some(api) = identity::riot_client_api() else {
        return false;
    };
    let Some(puuid) = api
        .get("/chat/v1/session")
        .await
        .ok()
        .and_then(|v| v.get("puuid").and_then(|p| p.as_str()).map(str::to_string))
    else {
        return false;
    };
    let Ok(v) = api.get("/chat/v4/presences").await else {
        return false;
    };
    v.get("presences")
        .and_then(|p| p.as_array())
        .into_iter()
        .flatten()
        .filter(|p| p.get("puuid").and_then(|x| x.as_str()) == Some(puuid.as_str()))
        .filter(|p| p.get("product").and_then(|x| x.as_str()) == Some("valorant"))
        .filter_map(|p| p.get("private").and_then(|x| x.as_str()).and_then(loop_state))
        .any(|s| s == "INGAME")
}

/// Complète un instantané de processus avec l'état de partie VALORANT (requête seulement si le jeu tourne).
pub async fn with_match_state(mut procs: ProcSnapshot) -> ProcSnapshot {
    if procs.val_game {
        procs.val_in_match = valorant_in_match().await;
    }
    procs
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b64(v: Value) -> String {
        base64::engine::general_purpose::STANDARD.encode(v.to_string())
    }

    #[test]
    fn reads_old_and_new_presence_formats() {
        let old = b64(serde_json::json!({"sessionLoopState": "INGAME"}));
        assert_eq!(loop_state(&old).as_deref(), Some("INGAME"));
        let new = b64(serde_json::json!({"matchPresenceData": {"sessionLoopState": "MENUS"}}));
        assert_eq!(loop_state(&new).as_deref(), Some("MENUS"));
        assert_eq!(loop_state("pas du base64 !"), None);
        assert_eq!(loop_state(&b64(serde_json::json!({"foo": 1}))), None);
    }
}
