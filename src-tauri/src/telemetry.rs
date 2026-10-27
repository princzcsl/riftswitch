//! Statistiques anonymes, opt-in, au plus une fois par jour, vers Aptabase (UE).
//! Le contenu envoyé est exactement celui de l'écran « Voir ce qui est envoyé ».

use tauri::{AppHandle, Manager};

use crate::state::AppState;
use crate::util::{log, now_ms};

/// Clé publique de l'app Aptabase (« A-EU-… »). Vide : rien n'est envoyé.
const APP_KEY: &str = "";

fn host(key: &str) -> Option<&'static str> {
    match key.split('-').nth(1)? {
        "EU" => Some("https://eu.aptabase.com"),
        "US" => Some("https://us.aptabase.com"),
        _ => None,
    }
}

fn today() -> String {
    (now_ms() / 86_400_000).to_string()
}

/// Envoie le résumé du jour si l'utilisateur l'a accepté et qu'il n'est pas déjà parti aujourd'hui.
pub async fn maybe_send(app: &AppHandle) {
    let Some(host) = host(APP_KEY) else { return };
    let state = app.state::<AppState>();
    let settings = state.settings();
    if !settings.telemetry || settings.telemetry_last_day == today() || !state.is_unlocked() {
        return;
    }
    let preview = crate::commands::app::build_preview(app, &state);
    let mut props = serde_json::Map::new();
    for (k, v) in preview.as_object().into_iter().flatten() {
        match v {
            // Aptabase n'accepte que des valeurs simples : les compteurs par jeu sont aplatis.
            serde_json::Value::Object(m) => {
                for (g, n) in m {
                    props.insert(format!("{k}_{g}"), n.clone());
                }
            }
            other => {
                props.insert(k.clone(), other.clone());
            }
        }
    }
    let event = serde_json::json!([{
        "timestamp": rfc3339(now_ms()),
        "sessionId": uuid::Uuid::new_v4().to_string(),
        "eventName": "daily",
        "systemProps": {
            "isDebug": cfg!(debug_assertions),
            "osName": std::env::consts::OS,
            "osVersion": "",
            "locale": settings.language,
            "appVersion": app.package_info().version.to_string(),
            "sdkVersion": "riftswitch",
        },
        "props": props,
    }]);
    let sent = reqwest::Client::new()
        .post(format!("{host}/api/v0/events"))
        .header("App-Key", APP_KEY)
        .json(&event)
        .send()
        .await
        .is_ok_and(|r| r.status().is_success());
    if sent {
        let mut s = state.settings.lock().unwrap();
        s.telemetry_last_day = today();
        let _ = s.save(&state.paths);
    }
    log(format!("telemetry: résumé du jour envoyé={sent}"));
}

/// Horodatage RFC 3339 (UTC) sans dépendance de date.
fn rfc3339(ms: i64) -> String {
    let secs = ms.div_euclid(1000);
    let (days, rem) = (secs.div_euclid(86_400), secs.rem_euclid(86_400));
    // Jours depuis 1970 → date civile (algorithme de Howard Hinnant).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}.{:03}Z",
        rem / 3600,
        rem % 3600 / 60,
        rem % 60,
        ms.rem_euclid(1000)
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn formats_rfc3339() {
        assert_eq!(super::rfc3339(0), "1970-01-01T00:00:00.000Z");
        assert_eq!(
            super::rfc3339(1_792_620_311_917),
            "2026-10-21T22:05:11.917Z"
        );
    }
}
