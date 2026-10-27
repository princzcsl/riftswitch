//! Connexion assistée aux comptes Riot (flux « clientless », d'après Ja-Sa-La/League-Account-Manager).
//!
//! RiftSwitch rejoue les identifiants que l'utilisateur a enregistrés (chiffrés) directement auprès
//! du service d'authentification officiel de Riot (`authenticate.riotgames.com`). Le captcha hCaptcha
//! est résolu par l'utilisateur (fenêtre de défi), l'A2F est saisie par l'utilisateur. En retour,
//! Riot émet un jeton de connexion, que l'on fait consommer par le Riot Client local.
//!
//! Aucun identifiant ni jeton n'est journalisé. La connexion remplace la saisie manuelle dans la
//! fenêtre du client ; c'est bien l'utilisateur qui s'authentifie sur ses propres comptes.

use serde_json::{json, Value};
use std::time::Duration;

use crate::error::AppResult;

const BASE: &str = "https://authenticate.riotgames.com";
pub const CLIENTLESS_HOST: &str = "authenticate.riotgames.com";
const USER_AGENT: &str =
    "RiotClient/70.0.0.247.1382 rso-authenticator (Windows;10;;Professional, x64)";

/// Exigence de défi (hCaptcha) renvoyée au démarrage de la session.
#[derive(Debug, Default, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Challenge {
    pub kind: String,
    pub site_key: Option<String>,
    pub rq_data: Option<String>,
}

impl Challenge {
    pub fn requires(&self) -> bool {
        !self.kind.eq_ignore_ascii_case("none") && self.site_key.is_some()
    }
}

/// Étape atteinte après une tentative de connexion.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase", tag = "step")]
pub enum Outcome {
    /// Connexion réussie : jeton de connexion à faire consommer par le client.
    Success { login_token: String },
    /// A2F requise : l'utilisateur doit fournir un code.
    Mfa {
        methods: Vec<String>,
        email: Option<String>,
    },
    /// Nouveau défi captcha requis (ex. captcha manqué).
    Challenge { challenge: Challenge },
    /// Échec (identifiants invalides, compte verrouillé…).
    Failed { reason: String },
}

/// Session de connexion : un client HTTP avec son propre bocal à cookies (asid, etc.).
pub struct LoginSession {
    client: reqwest::Client,
}

impl LoginSession {
    pub fn new() -> AppResult<Self> {
        let client = reqwest::Client::builder()
            .cookie_store(true)
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_secs(30))
            .user_agent(USER_AGENT)
            .build()?;
        Ok(Self { client })
    }

    async fn send(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<Value>,
    ) -> AppResult<Value> {
        let mut req = self
            .client
            .request(method, format!("{BASE}{path}"))
            .header("Accept", "application/json");
        if let Some(b) = body {
            req = req.json(&b);
        }
        let resp = req.send().await?;
        let text = resp.text().await.unwrap_or_default();
        Ok(serde_json::from_str(&text).unwrap_or(Value::Null))
    }

    /// Démarre une session d'authentification et renvoie l'éventuel défi captcha.
    pub async fn start(&self, remember: bool) -> AppResult<Challenge> {
        // DELETE réinitialise une session résiduelle, POST en démarre une neuve.
        let _ = self
            .send(reqwest::Method::DELETE, "/api/v1/login", None)
            .await;
        let body = json!({
            "apple": null, "campaign": null, "clientId": "riot-client", "code": null,
            "facebook": null, "gamecenter": null, "google": null, "language": "",
            "multifactor": null, "nintendo": null, "platform": "windows", "playstation": null,
            "remember": remember,
            "riot_identity": {
                "campaign": null, "captcha": null, "language": "en_US", "password": null,
                "remember": remember, "state": "auth", "username": null
            },
            "riot_identity_signup": null, "rso": null, "sdkVersion": "", "type": "auth", "xbox": null
        });
        let v = self
            .send(reqwest::Method::POST, "/api/v1/login", Some(body))
            .await?;
        Ok(parse_challenge(captcha_node(&v)))
    }

    /// Soumet identifiants + jeton captcha.
    pub async fn complete(
        &self,
        username: &str,
        password: &str,
        remember: bool,
        captcha_token: &str,
    ) -> AppResult<Outcome> {
        let body = json!({
            "campaign": null, "language": "en_US", "remember": remember,
            "riot_identity": {
                "captcha": format!("hcaptcha {captcha_token}"),
                "password": password, "state": null, "username": username
            },
            "type": "auth"
        });
        let v = self
            .send(reqwest::Method::PUT, "/api/v1/login", Some(body))
            .await?;
        let outcome = parse_outcome(&v);
        if !matches!(outcome, Outcome::Success { .. }) {
            crate::util::log(format!("login: réponse Riot {}", describe(&v)));
        }
        Ok(outcome)
    }

    /// Soumet un code d'A2F.
    pub async fn submit_mfa(&self, otp: &str, remember_device: bool) -> AppResult<Outcome> {
        let body = json!({
            "campaign": null, "language": "en_US", "remember": remember_device,
            "multifactor": { "action": null, "otp": otp, "rememberDevice": remember_device },
            "riot_identity": null, "type": "multifactor"
        });
        let v = self
            .send(reqwest::Method::PUT, "/api/v1/login", Some(body))
            .await?;
        Ok(parse_outcome(&v))
    }
}

fn str_of(v: &Value, k: &str) -> Option<String> {
    v.get(k)
        .and_then(|x| x.as_str())
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

fn captcha_node(v: &Value) -> Option<&Value> {
    v.get("captcha").or_else(|| v.get("challenge"))
}

fn parse_challenge(node: Option<&Value>) -> Challenge {
    let Some(c) = node else {
        return Challenge::default();
    };
    let kind = str_of(c, "type").unwrap_or_else(|| "none".into());
    let (mut site_key, mut rq_data) = (None, None);
    if let Some(hc) = c.get("hcaptcha") {
        site_key = str_of(hc, "key");
        rq_data = str_of(hc, "data");
    }
    Challenge {
        kind,
        site_key,
        rq_data,
    }
}

/// Cherche un `login_token` / `loginToken` n'importe où dans la réponse.
fn find_login_token(v: &Value) -> Option<String> {
    match v {
        Value::Object(m) => {
            for (k, val) in m {
                if (k == "login_token" || k == "loginToken") && val.is_string() {
                    return val.as_str().map(str::to_string);
                }
                if let Some(t) = find_login_token(val) {
                    return Some(t);
                }
            }
            None
        }
        Value::Array(a) => a.iter().find_map(find_login_token),
        _ => None,
    }
}

fn parse_outcome(v: &Value) -> Outcome {
    if let Some(token) = find_login_token(v) {
        return Outcome::Success { login_token: token };
    }
    let kind = str_of(v, "type").unwrap_or_default();
    if kind.eq_ignore_ascii_case("multifactor") {
        let mf = v.get("multifactor");
        let methods = mf
            .and_then(|m| m.get("methods"))
            .and_then(|x| x.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|x| x.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
        let email = mf.and_then(|m| str_of(m, "email"));
        return Outcome::Mfa { methods, email };
    }
    // Un refus contient aussi un nouveau captcha (pour réessayer) : l'erreur passe avant,
    // sinon chaque refus est pris pour une demande de captcha et on boucle.
    let error = error_of(v);
    let challenge = parse_challenge(captcha_node(v));
    if challenge.requires() && error.as_deref().map_or(true, |e| e.contains("captcha")) {
        return Outcome::Challenge { challenge };
    }
    let reason = error
        .or_else(|| str_of(v, "message"))
        .unwrap_or_else(|| "unknown".into());
    Outcome::Failed { reason }
}

/// Code d'erreur de Riot, à la racine ou dans `riot_identity`.
fn error_of(v: &Value) -> Option<String> {
    str_of(v, "error").or_else(|| v.get("riot_identity").and_then(|r| str_of(r, "error")))
}

/// Résumé sans valeur sensible d'une réponse non réussie, pour le journal.
fn describe(v: &Value) -> String {
    let keys = v
        .as_object()
        .map(|o| o.keys().cloned().collect::<Vec<_>>().join(","))
        .unwrap_or_default();
    format!(
        "type={} error={} champs=[{keys}]",
        str_of(v, "type").unwrap_or_default(),
        error_of(v).unwrap_or_default()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_challenge_from_start() {
        let v = json!({"type": "auth", "captcha": {"type": "hcaptcha", "hcaptcha": {"key": "sk", "data": "rq"}}});
        let c = parse_challenge(captcha_node(&v));
        assert!(c.requires());
        assert_eq!(c.site_key.as_deref(), Some("sk"));
        assert_eq!(c.rq_data.as_deref(), Some("rq"));
    }

    #[test]
    fn parses_success_mfa_and_failure() {
        let ok = parse_outcome(&json!({"type": "success", "success": {"login_token": "tok"}}));
        assert!(matches!(ok, Outcome::Success { login_token } if login_token == "tok"));

        let mfa = parse_outcome(
            &json!({"type": "multifactor", "multifactor": {"methods": ["email"], "email": "a***@b.com"}}),
        );
        assert!(matches!(mfa, Outcome::Mfa { .. }));

        let fail = parse_outcome(&json!({"type": "auth", "error": "auth_failure"}));
        assert!(matches!(fail, Outcome::Failed { reason } if reason == "auth_failure"));

        let chal = parse_outcome(
            &json!({"type": "auth", "captcha": {"type": "hcaptcha", "hcaptcha": {"key": "k"}}}),
        );
        assert!(matches!(chal, Outcome::Challenge { .. }));
        // Un refus arrive avec un nouveau captcha : c'est l'erreur qui compte.
        let captcha = json!({"type": "hcaptcha", "hcaptcha": {"key": "k"}});
        let refused =
            parse_outcome(&json!({"type": "auth", "error": "rate_limited", "captcha": captcha}));
        assert!(matches!(refused, Outcome::Failed { ref reason } if reason == "rate_limited"));
        let nested = parse_outcome(
            &json!({"type": "auth", "riot_identity": {"error": "auth_failure"}, "captcha": captcha}),
        );
        assert!(matches!(nested, Outcome::Failed { ref reason } if reason == "auth_failure"));
        let bad_captcha =
            parse_outcome(&json!({"type": "auth", "error": "captcha_failure", "captcha": captcha}));
        assert!(matches!(bad_captcha, Outcome::Challenge { .. }));
    }
}
