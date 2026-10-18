//! Surveillance de fond, volontairement sobre : un tour toutes les 4 s fenêtre ouverte, 15 s sinon.
//! - détecte le compte connecté au Riot Client (et met à jour Riot ID / anciens pseudos) ;
//! - lit rangs, decay et santé via le client LoL quand il est ouvert (cache dans le coffre) ;
//! - compte le temps de jeu par compte et par jeu ;
//! - verrouillage automatique (désactivé par défaut).

use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};

use crate::riot::{identity, lcu, paths, process, RiotStatus};
use crate::state::AppState;
use crate::util::{log, now_ms};
use crate::vault::Vault;

pub fn credit_playtime(vault: &mut Vault, account_id: &str, game: &str, secs: u64) {
    let Ok(idx) = vault.index_of(account_id) else {
        return;
    };
    let p = &mut vault.data.accounts[idx].playtime;
    match game {
        "val" => p.val += secs,
        "tft" => p.tft += secs,
        _ => p.lol += secs,
    }
    let day = (now_ms() / 86_400_000).to_string();
    *p.recent.entry(day).or_default() += secs;
    while p.recent.len() > 30 {
        let first = p.recent.keys().next().cloned();
        if let Some(k) = first {
            p.recent.remove(&k);
        }
    }
}

pub fn spawn(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut last_tick = Instant::now();
        let mut last_lcu: Option<Instant> = None;
        let mut ident: Option<identity::ActiveIdentity> = None;
        let mut ident_pid: Option<u32> = None;
        let mut ident_checked = Instant::now() - Duration::from_secs(600);
        let mut installed_checked = Instant::now() - Duration::from_secs(600);
        let mut installed = false;
        let mut pending_secs: u64 = 0;
        let mut pending_target: Option<(String, String)> = None;

        loop {
            let state = app.state::<AppState>();
            let visible = state.window_visible.load(Ordering::Relaxed);
            let interval = if visible { 4 } else { 15 };
            let settings = state.settings();
            let real_elapsed = last_tick.elapsed();
            last_tick = Instant::now();

            // Mise en veille probable (le tour a pris bien plus longtemps que prévu).
            if settings.lock_on_sleep
                && real_elapsed > Duration::from_secs(interval * 4 + 60)
                && state.is_unlocked()
            {
                log("monitor: retour de veille → verrouillage");
                crate::commands::app::do_lock(&app, &state);
            }

            if installed_checked.elapsed() > Duration::from_secs(120) {
                installed = paths::riot_client_services(&settings.riot_client_path).is_some();
                installed_checked = Instant::now();
            }

            let procs = tokio::task::spawn_blocking(process::snapshot)
                .await
                .unwrap_or_default();

            // ── Compte connecté au Riot Client ──
            if procs.riot_client {
                let stale = ident_checked.elapsed()
                    > Duration::from_secs(if ident.is_some() { 90 } else { 8 });
                if procs.riot_client_pid != ident_pid || stale {
                    ident = identity::active().await;
                    ident_pid = procs.riot_client_pid;
                    ident_checked = Instant::now();
                }
            } else {
                ident = None;
                ident_pid = None;
                state.client_via_proxy.store(false, Ordering::Relaxed);
            }

            let mut accounts_changed = false;
            let active_id = match &ident {
                Some(i) => state
                    .with_vault(|v| {
                        let Some(idx) = v.data.accounts.iter().position(|a| {
                            a.matches_identity(i.puuid.as_deref(), &i.game_name, &i.tag_line)
                        }) else {
                            return Ok(None);
                        };
                        let now = now_ms();
                        let a = &mut v.data.accounts[idx];
                        let before = (a.game_name.clone(), a.tag_line.clone(), a.puuid.clone());
                        a.observe_riot_id(&i.game_name, &i.tag_line, now);
                        if a.puuid.is_none() {
                            a.puuid = i.puuid.clone();
                        }
                        // Détails auto : région dès la connexion, shard VALORANT après un 1er lancement.
                        let mut filled = false;
                        if a.region.is_empty() {
                            if let Some(r) = &i.region {
                                a.region = r.clone();
                                filled = true;
                            }
                        }
                        if a.val_shard.is_none() {
                            if let Some(sh) =
                                a.puuid.as_deref().and_then(crate::riot::paths::valorant_shard)
                            {
                                a.val_shard = Some(sh);
                                filled = true;
                            }
                        }
                        let id = a.id.clone();
                        let dirty = filled
                            || before
                                != (a.game_name.clone(), a.tag_line.clone(), a.puuid.clone())
                            || v.data.last_active_account.as_deref() != Some(id.as_str());
                        v.data.last_active_account = Some(id.clone());
                        if dirty {
                            v.save()?;
                            accounts_changed = true;
                        }
                        Ok(Some(id))
                    })
                    .ok()
                    .flatten(),
                None => None,
            };

            // ── Rangs / decay / santé via le client LoL ──
            let force = state.force_sync.swap(false, Ordering::Relaxed);
            let due = last_lcu
                .map(|t| t.elapsed() > Duration::from_secs(180))
                .unwrap_or(true);
            if procs.lol_client && state.is_unlocked() && (force || due) {
                if let Some(api) = lcu::api(&settings.lol_path) {
                    if let Some(snap) = lcu::snapshot(&api, now_ms()).await {
                        last_lcu = Some(Instant::now());
                        let region =
                            api.get("/riotclient/region-locale")
                                .await
                                .ok()
                                .and_then(|v| {
                                    v.get("region").and_then(|x| x.as_str()).map(str::to_string)
                                });
                        let r = state.with_vault(|v| {
                            let Some(a) = v.data.accounts.iter_mut().find(|a| {
                                a.matches_identity(
                                    snap.puuid.as_deref(),
                                    &snap.game_name,
                                    &snap.tag_line,
                                )
                            }) else {
                                return Ok(false);
                            };
                            if !a.ranks.manual || a.ranks.val.is_none() {
                                let val = a.ranks.val.take();
                                a.ranks = snap.ranks.clone();
                                a.ranks.val = val;
                            } else {
                                a.ranks.lol_solo = snap.ranks.lol_solo.clone();
                                a.ranks.lol_flex = snap.ranks.lol_flex.clone();
                                a.ranks.tft = snap.ranks.tft.clone();
                                a.ranks.updated_at = snap.ranks.updated_at;
                            }
                            a.decay = snap.decay.clone();
                            a.health = snap.health.clone();
                            a.level = snap.level.or(a.level);
                            a.profile_icon_id = snap.profile_icon_id.or(a.profile_icon_id);
                            if a.puuid.is_none() {
                                a.puuid = snap.puuid.clone();
                            }
                            if a.region.is_empty() {
                                if let Some(r) = &region {
                                    a.region = r.clone();
                                }
                            }
                            v.save()?;
                            Ok(true)
                        });
                        if r.unwrap_or(false) {
                            accounts_changed = true;
                        }
                    }
                }
            }

            // ── Temps de jeu ──
            if procs.lol_game || procs.val_game {
                let fallback = state
                    .with_vault(|v| Ok(v.data.last_active_account.clone()))
                    .ok()
                    .flatten();
                if let Some(id) = active_id.clone().or(fallback) {
                    let game = if procs.val_game {
                        "val".to_string()
                    } else {
                        let last = state
                            .with_vault(|v| {
                                Ok(v.data
                                    .accounts
                                    .iter()
                                    .find(|a| a.id == id)
                                    .and_then(|a| a.last_game.clone()))
                            })
                            .ok()
                            .flatten();
                        if last.as_deref() == Some("tft") {
                            "tft".into()
                        } else {
                            "lol".into()
                        }
                    };
                    pending_target = Some((id, game));
                    pending_secs += real_elapsed.as_secs().min(90);
                }
            }
            let game_over = !(procs.lol_game || procs.val_game);
            if pending_secs > 0 && (game_over || pending_secs >= 120) {
                if let Some((id, game)) = pending_target.clone() {
                    let flushed = state
                        .with_vault(|v| {
                            credit_playtime(v, &id, &game, pending_secs);
                            v.save()
                        })
                        .is_ok();
                    if !flushed {
                        state
                            .pending_playtime
                            .lock()
                            .unwrap()
                            .push((id, game, pending_secs));
                    }
                    pending_secs = 0;
                    accounts_changed = true;
                }
            }

            // ── Verrouillage automatique ──
            if settings.auto_lock_minutes > 0 && state.is_unlocked() {
                let idle = now_ms() - state.last_activity.load(Ordering::Relaxed);
                if idle > settings.auto_lock_minutes as i64 * 60_000 {
                    log("monitor: verrouillage automatique");
                    crate::commands::app::do_lock(&app, &state);
                }
            }

            if accounts_changed {
                let _ = app.emit("accounts-changed", ());
            }

            // ── Statut publié vers l'UI (seulement s'il change) ──
            let proxy = state.proxy();
            let next = {
                let current = state.status.lock().unwrap();
                RiotStatus {
                    client_installed: installed,
                    procs: procs.clone(),
                    identity: ident.clone(),
                    active_account_id: active_id.clone(),
                    proxy_running: proxy.is_some()
                        && state.client_via_proxy.load(Ordering::Relaxed),
                    offline: proxy.as_ref().is_some_and(|p| p.is_offline())
                        && state.client_via_proxy.load(Ordering::Relaxed),
                    busy: current.busy.clone(),
                }
            };
            let changed = {
                let mut current = state.status.lock().unwrap();
                let c = *current != next;
                *current = next.clone();
                c
            };
            if changed && visible {
                let _ = app.emit("riot-status", next);
            }

            tokio::time::sleep(Duration::from_secs(interval)).await;
        }
    });
}
