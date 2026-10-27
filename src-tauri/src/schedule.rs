//! Lancements planifiés : à l'heure dite, bascule sur le compte et lancement du jeu choisi.
//! Jamais pendant qu'un jeu est ouvert : le lancement est alors sauté.

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};

use crate::riot::process;
use crate::state::AppState;
use crate::util::log;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", default)]
pub struct Schedule {
    pub id: String,
    pub account_id: String,
    pub game: String,
    pub offline: bool,
    /// Heure locale « HH:MM ».
    pub time: String,
    /// Jours de la semaine, 0 = dimanche.
    pub days: Vec<u8>,
    pub enabled: bool,
    /// Dernier jour d'exécution « AAAA-MM-JJ » (une fois par jour au plus).
    pub last_run: String,
}

/// Instant local : (jour de la semaine, « HH:MM », « AAAA-MM-JJ »).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalNow {
    pub weekday: u8,
    pub time: String,
    pub date: String,
}

#[cfg(windows)]
pub fn local_now() -> LocalNow {
    let t = unsafe { windows::Win32::System::SystemInformation::GetLocalTime() };
    LocalNow {
        weekday: t.wDayOfWeek as u8,
        time: format!("{:02}:{:02}", t.wHour, t.wMinute),
        date: format!("{:04}-{:02}-{:02}", t.wYear, t.wMonth, t.wDay),
    }
}

#[cfg(not(windows))]
pub fn local_now() -> LocalNow {
    // Repli sans fuseau horaire (UTC) hors Windows.
    let secs = crate::util::now_ms() / 1000;
    let days = secs / 86_400;
    LocalNow {
        weekday: ((days + 4) % 7) as u8,
        time: format!("{:02}:{:02}", secs % 86_400 / 3600, secs % 3600 / 60),
        date: days.to_string(),
    }
}

pub fn is_due(s: &Schedule, now: &LocalNow) -> bool {
    s.enabled && s.days.contains(&now.weekday) && s.time == now.time && s.last_run != now.date
}

/// Appelé à chaque tour du moniteur.
pub fn tick(app: &AppHandle) {
    let state = app.state::<AppState>();
    let now = local_now();
    let due: Vec<Schedule> = state
        .settings()
        .schedules
        .iter()
        .filter(|s| is_due(s, &now))
        .cloned()
        .collect();
    let Some(first) = due.first().cloned() else {
        return;
    };
    {
        let mut settings = state.settings.lock().unwrap();
        for s in settings.schedules.iter_mut().filter(|s| due.iter().any(|d| d.id == s.id)) {
            s.last_run = now.date.clone();
        }
        let _ = settings.save(&state.paths);
    }
    if !state.is_unlocked() {
        log("schedule: lancement sauté, coffre verrouillé");
        return;
    }
    let procs = process::snapshot();
    if procs.lol_client || procs.lol_game || procs.val_game {
        log("schedule: lancement sauté, un jeu est déjà ouvert");
        return;
    }
    log(format!("schedule: lancement planifié de {} à {}", first.game, now.time));
    let _ = app.emit("schedule-run", first.account_id.clone());
    crate::commands::play::play_external(app.clone(), first.account_id, first.game, first.offline);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn due_once_per_day_on_selected_days() {
        let s = Schedule {
            time: "21:00".into(),
            days: vec![1, 3],
            enabled: true,
            ..Default::default()
        };
        let now = LocalNow { weekday: 1, time: "21:00".into(), date: "2026-10-26".into() };
        assert!(is_due(&s, &now));
        assert!(!is_due(&Schedule { last_run: "2026-10-26".into(), ..s.clone() }, &now));
        assert!(!is_due(&s, &LocalNow { weekday: 2, ..now.clone() }));
        assert!(!is_due(&s, &LocalNow { time: "21:01".into(), ..now.clone() }));
        assert!(!is_due(&Schedule { enabled: false, ..s }, &now));
    }
}
