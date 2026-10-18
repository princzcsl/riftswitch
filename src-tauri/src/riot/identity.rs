//! Identité du compte connecté au Riot Client (API locale). Plusieurs routes sont tentées car elles
//! varient selon les versions du client ; chacune peut renvoyer un objet ou un tableau.

use serde::Serialize;
use serde_json::Value;

use super::local_api::LocalApi;
use super::paths;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActiveIdentity {
    pub game_name: String,
    pub tag_line: String,
    pub puuid: Option<String>,
    /// Région LoL du compte (ex. « EUW »), connue dès la connexion, même sans lancer de jeu.
    pub region: Option<String>,
}

// userinfo en premier : une seule requête donne Riot ID, PUUID et région.
const ENDPOINTS: [&str; 3] = [
    "/rso-auth/v1/authorization/userinfo",
    "/player-account/aliases/v1/active",
    "/chat/v1/session",
];

/// Plateforme Riot (`EUW1`, `LA2`…) vers la région affichée dans RiftSwitch.
pub fn platform_region(platform: &str) -> Option<&'static str> {
    let p = platform.trim().to_ascii_uppercase();
    Some(match p.trim_end_matches(|c: char| c.is_ascii_digit()) {
        "EUW" => "EUW",
        "EUN" => "EUNE",
        "NA" => "NA",
        "KR" => "KR",
        "JP" => "JP",
        "BR" => "BR",
        "LA" if p == "LA1" => "LAN",
        "LA" => "LAS",
        "OC" => "OCE",
        "TR" => "TR",
        "RU" => "RU",
        "ME" => "ME",
        "SG" | "PH" | "TH" => "SEA",
        "TW" => "TW",
        "VN" => "VN",
        _ => return None,
    })
}

fn region_of(v: &Value) -> Option<String> {
    let platform = v
        .get("lol")
        .and_then(|l| s(l, "cpid").or_else(|| s(l, "pid")))
        .or_else(|| {
            v.get("lol_region")
                .and_then(|r| r.as_array())
                .and_then(|a| a.iter().find_map(|x| s(x, "cpid")))
        })
        .or_else(|| s(v, "original_platform_id"))?;
    platform_region(&platform).map(str::to_string)
}

pub fn riot_client_api() -> Option<LocalApi> {
    LocalApi::from_lockfile(&paths::riot_client_lockfile())
}

pub async fn active() -> Option<ActiveIdentity> {
    let api = riot_client_api()?;
    let mut found: Option<ActiveIdentity> = None;
    for ep in ENDPOINTS {
        if let Ok(v) = api.get(ep).await {
            if let Some(id) = parse(&v) {
                // Certaines routes n'exposent pas tout : on complète avec les suivantes.
                match &mut found {
                    None => found = Some(id),
                    Some(f) => {
                        f.puuid = f.puuid.take().or(id.puuid);
                        f.region = f.region.take().or(id.region);
                    }
                }
                if found
                    .as_ref()
                    .is_some_and(|f| f.puuid.is_some() && f.region.is_some())
                {
                    break;
                }
            }
        }
    }
    found
}

fn s(v: &Value, k: &str) -> Option<String> {
    v.get(k)
        .and_then(|x| x.as_str())
        .filter(|x| !x.is_empty())
        .map(str::to_string)
}

pub fn parse(v: &Value) -> Option<ActiveIdentity> {
    if let Some(arr) = v.as_array() {
        return arr.iter().find_map(parse);
    }
    if !v.is_object() {
        return None;
    }
    // userinfo renvoie parfois un sous-objet JSON encodé en chaîne.
    if let Some(inner) = v.get("userInfo").and_then(|x| x.as_str()) {
        if let Ok(parsed) = serde_json::from_str::<Value>(inner) {
            if let Some(id) = parse(&parsed) {
                return Some(id);
            }
        }
    }
    let mut game_name = s(v, "game_name").or_else(|| s(v, "gameName"));
    let mut tag_line = s(v, "tag_line").or_else(|| s(v, "tagLine"));
    if game_name.is_none() {
        if let Some(acct) = v.get("acct") {
            game_name = s(acct, "game_name");
            tag_line = s(acct, "tag_line");
        }
    }
    let tag_line = tag_line.or_else(|| s(v, "game_tag")).unwrap_or_default();
    let puuid = s(v, "puuid").or_else(|| s(v, "sub"));
    Some(ActiveIdentity {
        game_name: game_name?,
        tag_line,
        puuid,
        region: region_of(v),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_known_shapes() {
        let a = parse(&json!([{"game_name": "Prince", "tag_line": "EUW"}])).unwrap();
        assert_eq!(
            (a.game_name.as_str(), a.tag_line.as_str()),
            ("Prince", "EUW")
        );
        let b = parse(&json!({"game_name": "X", "game_tag": "123", "puuid": "p"})).unwrap();
        assert_eq!(b.tag_line, "123");
        assert_eq!(b.puuid.as_deref(), Some("p"));
        let inner = json!({"sub": "pp", "acct": {"game_name": "Y", "tag_line": "T"}}).to_string();
        let c = parse(&json!({"userInfo": inner})).unwrap();
        assert_eq!(
            (c.game_name.as_str(), c.puuid.as_deref()),
            ("Y", Some("pp"))
        );
        assert!(parse(&json!({"foo": 1})).is_none());
    }

    #[test]
    fn reads_region_from_userinfo() {
        let inner = json!({
            "sub": "pp",
            "acct": {"game_name": "Y", "tag_line": "T"},
            "lol": {"cpid": "EUN1"},
        })
        .to_string();
        let a = parse(&json!({"userInfo": inner})).unwrap();
        assert_eq!(a.region.as_deref(), Some("EUNE"));
        let b = parse(&json!({"game_name": "X", "lol_region": [{"cpid": "LA2"}]})).unwrap();
        assert_eq!(b.region.as_deref(), Some("LAS"));
        assert_eq!(platform_region("LA1"), Some("LAN"));
        assert_eq!(platform_region("PBE1"), None);
        assert!(parse(&json!({"game_name": "X"})).unwrap().region.is_none());
    }
}
