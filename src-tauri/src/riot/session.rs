//! Session mémorisée du Riot Client et jetons RSO propres à RiftSwitch.
//!
//! Ce que le Riot Client restaure au démarrage, c'est l'enregistrement
//! `psl.authorization.riot-client` de `Data/RiotGamesPrivateSettings.yaml` (refresh token + id token).
//! Ces refresh tokens sont consommés à l'usage : copier le fichier ne marche donc pas de façon fiable.
//!
//! Technique (inspirée de Ja-Sa-La/League-Account-Manager) : pour chaque compte, RiftSwitch obtient
//! SA PROPRE paire de jetons, indépendante de celle du client :
//! 1. le client connecté émet un jeton de connexion à usage unique
//!    (`POST /rso-authenticator/v1/authentication/redirect`) ;
//! 2. ce jeton est échangé chez Riot (`auth.riotgames.com/token`, échange RFC 8693) contre un
//!    refresh token + id token, chiffrés dans le coffre ;
//! 3. pour basculer, ces jetons sont écrits dans l'enregistrement `psl` (le reste du fichier,
//!    dont l'identifiant d'appareil `tdid`, est conservé), puis le client démarre et se reconnecte ;
//! 4. une fois connecté, le client a consommé notre refresh token : on en ré-émet un aussitôt.
//!
//! Aucune valeur de jeton n'est jamais journalisée.

use base64::engine::general_purpose::URL_SAFE_NO_PAD as B64URL;
use base64::Engine;
use serde_json::Value;
use std::fs;
use std::time::Duration;
use zeroize::Zeroizing;

use super::local_api::LocalApi;
use super::paths::{riot_client_data, SESSION_FILES};
use crate::error::{AppError, AppResult};
use crate::util::{self, now_ms};

const AUTH_TOKEN_URL: &str = "https://auth.riotgames.com/token";
const RSO_USER_AGENT: &str = "RiotClient/70.0.0.247.1382 rso-auth (Windows;10;;Professional, x64)";
/// Portée d'une session mémorisée : sans `offline_access`, le refresh token est court et refusé ensuite.
const EXCHANGE_SCOPE: &str = "openid link ban lol_region lol account summoner offline_access";
const PERSISTED_SCOPES: [&str; 6] = ["openid", "link", "ban", "lol_region", "lol", "account"];

/// Paire de jetons RSO d'un compte (toujours manipulée dans des `Zeroizing`).
pub struct RsoTokens {
    pub refresh_token: Zeroizing<String>,
    pub id_token: Zeroizing<String>,
}

impl RsoTokens {
    /// PUUID du compte, lu dans l'id token (champ `sub` du JWT, sans vérification de signature :
    /// sert uniquement à rattacher le jeton au bon compte).
    pub fn puuid(&self) -> Option<String> {
        jwt_sub(&self.id_token)
    }
}

pub fn jwt_sub(jwt: &str) -> Option<String> {
    let payload = jwt.split('.').nth(1)?;
    let bytes = B64URL.decode(payload.trim_end_matches('=')).ok()?;
    let v: Value = serde_json::from_slice(&bytes).ok()?;
    v.get("sub").and_then(|s| s.as_str()).map(str::to_string)
}

fn find_login_token(v: &Value) -> Option<String> {
    match v {
        Value::Object(map) => {
            for (k, val) in map {
                if (k == "login_token" || k == "loginToken") && val.is_string() {
                    return val.as_str().map(str::to_string);
                }
                if let Some(t) = find_login_token(val) {
                    return Some(t);
                }
            }
            None
        }
        Value::Array(arr) => arr.iter().find_map(find_login_token),
        _ => None,
    }
}

/// Émet un jeton de connexion à usage unique depuis la session actuellement connectée du client.
pub async fn mint_login_token(api: &LocalApi) -> AppResult<Zeroizing<String>> {
    let v = api
        .post(
            "/rso-authenticator/v1/authentication/redirect",
            &serde_json::json!({}),
        )
        .await?;
    find_login_token(&v)
        .map(Zeroizing::new)
        .ok_or_else(|| AppError::new("not_signed_in", "no login token"))
}

/// Échange un jeton de connexion contre une paire refresh token / id token propre à RiftSwitch.
pub async fn exchange(login_token: &str) -> AppResult<RsoTokens> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .build()?;
    let form = [
        ("client_id", "riot-client"),
        (
            "grant_type",
            "urn:ietf:params:oauth:grant-type:token-exchange",
        ),
        ("subject_token", login_token),
        (
            "subject_token_type",
            "urn:riot:params:oauth:token-type:authentication-token",
        ),
        ("scope", EXCHANGE_SCOPE),
    ];
    let resp = client
        .post(AUTH_TOKEN_URL)
        .header("User-Agent", RSO_USER_AGENT)
        .form(&form)
        .send()
        .await?;
    let status = resp.status().as_u16();
    let text = Zeroizing::new(resp.text().await.unwrap_or_default());
    if status != 200 {
        return Err(AppError::new("token_exchange", format!("HTTP {status}")));
    }
    let v: Value =
        serde_json::from_str(&text).map_err(|_| AppError::new("token_exchange", "json"))?;
    let refresh = v.get("refresh_token").and_then(|x| x.as_str());
    let id = v.get("id_token").and_then(|x| x.as_str());
    match (refresh, id) {
        (Some(r), Some(i)) if !r.is_empty() && !i.is_empty() => Ok(RsoTokens {
            refresh_token: Zeroizing::new(r.to_string()),
            id_token: Zeroizing::new(i.to_string()),
        }),
        _ => Err(AppError::new("token_exchange", "missing tokens")),
    }
}

/// Émet puis échange : nouvelle paire de jetons pour le compte connecté au client.
pub async fn mint_tokens(api: &LocalApi) -> AppResult<RsoTokens> {
    let login = mint_login_token(api).await?;
    exchange(&login).await
}

fn settings_path() -> std::path::PathBuf {
    riot_client_data().join(SESSION_FILES[0])
}

fn persisted_record(tokens: &RsoTokens) -> String {
    let now = now_ms();
    let mut s = String::new();
    s.push_str("psl:\n");
    s.push_str("    authorization:\n");
    s.push_str("        riot-client:\n");
    s.push_str("            claims: []\n");
    s.push_str(&format!("            id_token: \"{}\"\n", *tokens.id_token));
    s.push_str("            is_dpop_bound: false\n");
    s.push_str(&format!("            last_token_creation_time: {now}\n"));
    s.push_str(&format!(
        "            original_token_creation_time: {now}\n"
    ));
    s.push_str(&format!(
        "            refresh_token: \"{}\"\n",
        *tokens.refresh_token
    ));
    s.push_str("            refresh_token_write_count: 1\n");
    s.push_str(&format!(
        "            refresh_tokens_session_id: \"{}\"\n",
        uuid::Uuid::new_v4()
    ));
    s.push_str("            scopes:\n");
    for scope in PERSISTED_SCOPES {
        s.push_str(&format!("            - \"{scope}\"\n"));
    }
    s
}

/// Remplace le bloc de premier niveau `psl:` (jusqu'à la clé de premier niveau suivante) par
/// `record` ; sans bloc existant, `record` est ajouté en tête. Les autres blocs sont conservés.
pub fn replace_psl_block(yaml: &str, record: &str) -> String {
    let lines: Vec<&str> = yaml.lines().collect();
    let start = lines.iter().position(|l| l.starts_with("psl:"));
    let Some(start) = start else {
        return if yaml.trim().is_empty() {
            record.to_string()
        } else {
            format!("{record}{yaml}")
        };
    };
    let end = lines[start + 1..]
        .iter()
        .position(|l| {
            !l.is_empty() && !l.starts_with(' ') && !l.starts_with('\t') && !l.starts_with('-')
        })
        .map(|p| start + 1 + p)
        .unwrap_or(lines.len());
    let mut out = String::new();
    for l in &lines[..start] {
        out.push_str(l);
        out.push('\n');
    }
    out.push_str(record);
    for l in &lines[end..] {
        out.push_str(l);
        out.push('\n');
    }
    out
}

/// Écrit les jetons du compte cible : le client se connectera avec au prochain démarrage.
/// Le client doit être fermé (il réécrit le fichier depuis sa mémoire tant qu'il tourne).
pub fn write_persisted(tokens: &RsoTokens) -> AppResult<()> {
    let path = settings_path();
    let current = Zeroizing::new(fs::read_to_string(&path).unwrap_or_default());
    let record = Zeroizing::new(persisted_record(tokens));
    let updated = Zeroizing::new(replace_psl_block(&current, &record));
    util::atomic_write(&path, updated.as_bytes(), false)?;
    Ok(())
}

/// Retire la session mémorisée (bloc `psl`) en gardant l'identifiant d'appareil : le client démarrera
/// sur l'écran de connexion officiel, sans être vu comme un nouvel appareil.
pub fn clear_persisted() -> AppResult<()> {
    let path = settings_path();
    let Ok(current) = fs::read_to_string(&path) else {
        return Ok(());
    };
    let current = Zeroizing::new(current);
    let updated = replace_psl_block(&current, "");
    util::atomic_write(&path, updated.as_bytes(), false)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const FILE: &str = "psl:\n    authorization:\n        riot-client:\n            refresh_token: \"old\"\n            scopes:\n            - \"openid\"\nrso-authenticator:\n    tdid:\n        value: \"device\"\n";

    fn tokens() -> RsoTokens {
        // id token factice : {"sub":"puuid-123"}
        let payload = B64URL.encode(br#"{"sub":"puuid-123"}"#);
        RsoTokens {
            refresh_token: Zeroizing::new("newrefresh".into()),
            id_token: Zeroizing::new(format!("hdr.{payload}.sig")),
        }
    }

    #[test]
    fn replaces_only_psl_block() {
        let rec = persisted_record(&tokens());
        let out = replace_psl_block(FILE, &rec);
        assert!(out.contains("refresh_token: \"newrefresh\""));
        assert!(!out.contains("\"old\""));
        assert!(out.contains("rso-authenticator:\n    tdid:\n        value: \"device\""));
        assert_eq!(out.matches("psl:").count(), 1);
    }

    #[test]
    fn clear_keeps_device_id() {
        let out = replace_psl_block(FILE, "");
        assert!(!out.contains("psl:"));
        assert!(out.contains("tdid"));
    }

    #[test]
    fn adds_block_when_missing() {
        let rec = persisted_record(&tokens());
        let out = replace_psl_block("rso-authenticator:\n    tdid: x\n", &rec);
        assert!(out.starts_with("psl:"));
        assert!(out.contains("tdid: x"));
        assert!(replace_psl_block("", &rec).starts_with("psl:"));
    }

    #[test]
    fn reads_puuid_from_id_token() {
        assert_eq!(tokens().puuid().as_deref(), Some("puuid-123"));
        assert_eq!(jwt_sub("not-a-jwt"), None);
    }

    #[test]
    fn finds_login_token_anywhere() {
        let v = serde_json::json!({"type": "success", "success": {"login_token": "abc"}});
        assert_eq!(find_login_token(&v).as_deref(), Some("abc"));
        let v = serde_json::json!({"data": [{"loginToken": "x"}]});
        assert_eq!(find_login_token(&v).as_deref(), Some("x"));
        assert_eq!(find_login_token(&serde_json::json!({"a": 1})), None);
    }
}
