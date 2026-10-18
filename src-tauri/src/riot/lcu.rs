//! API locale du client League of Legends (LCU) : rangs, decay, santé du compte, réglages, runes.
//! Toutes les lectures sont « au mieux » : un champ absent ne casse rien.

use serde::Serialize;
use serde_json::Value;

use super::local_api::LocalApi;
use super::paths;
use crate::vault::model::{Decay, Health, Rank, Ranks};

pub fn api(lol_custom: &str) -> Option<LocalApi> {
    LocalApi::from_lockfile(&paths::lcu_lockfile(lol_custom)?)
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LolSnapshot {
    pub game_name: String,
    pub tag_line: String,
    pub puuid: Option<String>,
    pub level: Option<i64>,
    pub profile_icon_id: Option<i64>,
    pub ranks: Ranks,
    pub decay: Decay,
    pub health: Health,
}

fn str_of(v: &Value, k: &str) -> String {
    v.get(k)
        .and_then(|x| x.as_str())
        .unwrap_or_default()
        .to_string()
}

fn rank_of(q: &Value) -> Option<Rank> {
    let tier = str_of(q, "tier");
    if tier.is_empty() || tier == "NONE" {
        return None;
    }
    let division = str_of(q, "division");
    Some(Rank {
        tier,
        division: if division == "NA" {
            String::new()
        } else {
            division
        },
        lp: q.get("leaguePoints").and_then(|x| x.as_i64()).unwrap_or(0) as i32,
        wins: q.get("wins").and_then(|x| x.as_i64()).unwrap_or(0) as i32,
        losses: q.get("losses").and_then(|x| x.as_i64()).unwrap_or(0) as i32,
    })
}

pub async fn snapshot(api: &LocalApi, now: i64) -> Option<LolSnapshot> {
    let summoner = api.get("/lol-summoner/v1/current-summoner").await.ok()?;
    let mut snap = LolSnapshot {
        game_name: str_of(&summoner, "gameName"),
        tag_line: str_of(&summoner, "tagLine"),
        puuid: summoner
            .get("puuid")
            .and_then(|x| x.as_str())
            .map(str::to_string),
        level: summoner.get("summonerLevel").and_then(|x| x.as_i64()),
        profile_icon_id: summoner.get("profileIconId").and_then(|x| x.as_i64()),
        ..Default::default()
    };

    if let Ok(ranked) = api.get("/lol-ranked/v1/current-ranked-stats").await {
        if let Some(queues) = ranked.get("queues").and_then(|q| q.as_array()) {
            for q in queues {
                let r = rank_of(q);
                match q.get("queueType").and_then(|x| x.as_str()) {
                    Some("RANKED_SOLO_5x5") => {
                        snap.ranks.lol_solo = r;
                        if let Some(w) = q.get("warnings").filter(|w| w.is_object()) {
                            snap.decay.days_remaining = w
                                .get("daysUntilDecay")
                                .and_then(|x| x.as_i64())
                                .map(|d| d as i32);
                            snap.decay.banked_games = w
                                .get("bankedGames")
                                .and_then(|x| x.as_i64())
                                .map(|d| d as i32);
                        }
                    }
                    Some("RANKED_FLEX_SR") => snap.ranks.lol_flex = r,
                    Some("RANKED_TFT") => snap.ranks.tft = r,
                    _ => {}
                }
            }
        }
        snap.ranks.updated_at = Some(now);
        snap.decay.updated_at = Some(now);
    }

    // Santé du compte : routes variables selon les versions, lectures facultatives.
    if let Ok(v) = api.get("/lol-honor-v2/v1/profile").await {
        snap.health.honor_level = v
            .get("honorLevel")
            .and_then(|x| x.as_i64())
            .map(|h| h as i32);
    }
    if let Ok(v) = api.get("/lol-player-behavior/v1/ranked-restriction").await {
        let games = v
            .get("punishedGamesRemaining")
            .and_then(|x| x.as_i64())
            .unwrap_or(0);
        if games > 0 {
            snap.health
                .penalties
                .push(format!("ranked_restriction:{games}"));
        }
    }
    if let Ok(v) = api.get("/lol-leaver-buster/v1/ranked-restriction").await {
        let games = v
            .get("punishedGamesRemaining")
            .and_then(|x| x.as_i64())
            .unwrap_or(0);
        if games > 0 && snap.health.penalties.is_empty() {
            snap.health
                .penalties
                .push(format!("ranked_restriction:{games}"));
        }
    }
    if let Ok(v) = api.get("/lol-matchmaking/v1/search/errors").await {
        if let Some(arr) = v.as_array() {
            for e in arr {
                let t = str_of(e, "penaltyTimeRemaining");
                if !t.is_empty()
                    || e.get("penaltyTimeRemaining")
                        .and_then(|x| x.as_f64())
                        .unwrap_or(0.0)
                        > 0.0
                {
                    snap.health.penalties.push("queue_lockout".into());
                }
            }
        }
    }
    snap.health.updated_at = Some(now);
    Some(snap)
}

/// L'utilisateur est-il connecté au client LoL (pas seulement le client lancé) ?
pub async fn logged_in(api: &LocalApi) -> bool {
    api.get("/lol-summoner/v1/current-summoner")
        .await
        .map(|v| v.get("puuid").is_some())
        .unwrap_or(false)
}
