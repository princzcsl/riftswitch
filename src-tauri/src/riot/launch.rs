//! Pilotage du Riot Client : démarrage, arrêt complet, attente de l'authentification, lancement des jeux.
//!
//! Règle apprise sur le vrai client : quand il tourne déjà, relancer `RiotClientServices.exe` avec
//! `--launch-product` crée un second processus qui n'arrive pas à joindre le premier
//! (`ProcessSingleton … timed out`) et démarre seul, déconnecté. On lance donc les jeux via l'API
//! locale du client (`product-launcher`), et on ne démarre l'exécutable que si aucun client ne tourne.

use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

use super::{identity, paths, process};
use crate::error::{AppError, AppResult};
use crate::util::log;

/// Démarre le Riot Client. `game` ajoute le produit à préparer (comme LAM : le client démarre déjà
/// sur le bon produit, prêt à être connecté) ; `extra` sert au mode hors ligne (`--client-config-url`).
pub fn start_client(exe: &Path, game: Option<&str>, extra: &[String]) -> AppResult<()> {
    let mut cmd = Command::new(exe);
    if let Some(dir) = exe.parent() {
        cmd.current_dir(dir);
    }
    if let Some(g) = game {
        cmd.arg(format!("--launch-product={}", paths::product_id(g)));
        cmd.arg("--launch-patchline=live");
    }
    cmd.args(extra);
    let child = cmd.spawn().map_err(|e| {
        log(format!("client: échec du démarrage {e}"));
        AppError::new("launch_failed", e.to_string())
    })?;
    log(format!(
        "client: démarré pid={} jeu={game:?} extra={}",
        child.id(),
        extra.len()
    ));
    Ok(())
}

fn lockfile_present() -> bool {
    paths::riot_client_lockfile().exists()
}

/// Ferme complètement le Riot Client et les jeux : demande d'arrêt propre (le client écrit alors sa
/// session sur disque), attente de la disparition des processus ET du lockfile, sinon arrêt forcé.
pub async fn shutdown(timeout: Duration) {
    if let Some(api) = identity::riot_client_api() {
        let _ = api
            .post("/process-control/v1/process/quit", &serde_json::Value::Null)
            .await;
    }
    let start = Instant::now();
    let mut clean = false;
    while start.elapsed() < timeout {
        if !process::snapshot().any_riot() && !lockfile_present() {
            clean = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(300)).await;
    }
    if !clean {
        let killed = process::kill_all();
        log(format!("client: {killed} processus arrêtés de force"));
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(5) && process::snapshot().any_riot() {
            tokio::time::sleep(Duration::from_millis(300)).await;
        }
    } else {
        log(format!(
            "client: fermé proprement en {} ms",
            start.elapsed().as_millis()
        ));
    }
    // Marge : le client finalise parfois ses fichiers juste après la sortie.
    tokio::time::sleep(Duration::from_millis(700)).await;
}

/// Attend que l'API locale du client réponde (le lockfile existe et une requête aboutit).
pub async fn wait_api(timeout: Duration) -> bool {
    let start = Instant::now();
    while start.elapsed() < timeout {
        if let Some(api) = identity::riot_client_api() {
            if api.status("/rso-auth/v1/authorization").await.is_some() {
                return true;
            }
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    false
}

/// Le client a-t-il une session authentifiée ?
pub async fn authenticated() -> bool {
    match identity::riot_client_api() {
        Some(api) => api.status("/rso-auth/v1/authorization").await == Some(200),
        None => false,
    }
}

/// Lance un jeu via l'API locale du client (comme le bouton Jouer du Riot Client).
/// Réessaie le temps que le lanceur de produits soit prêt, puis se rabat sur la ligne de commande.
pub async fn launch_product(exe: &Path, game: &str, locale: Option<&str>) -> AppResult<()> {
    let product = paths::product_id(game);
    let route = format!("/product-launcher/v1/products/{product}/patchlines/live");
    let start = Instant::now();
    let mut last = String::new();
    // 423 = le client lance déjà ce produit (démarré avec `--launch-product`, il lance le jeu tout
    // seul après la connexion) : on cesse de le redemander et on attend simplement son ouverture.
    let mut in_progress = false;
    let limit = |busy: bool| Duration::from_secs(if busy { 30 } else { 12 });
    while start.elapsed() < limit(in_progress) {
        if super::process::snapshot().product_running(game) {
            log(format!(
                "launch: {product} ouvert en {} ms",
                start.elapsed().as_millis()
            ));
            return Ok(());
        }
        if !in_progress {
            if let Some(api) = identity::riot_client_api() {
                // Même appel que le bouton Jouer officiel : POST sans corps.
                match api.request(reqwest::Method::POST, &route, None).await {
                    Ok(_) => {
                        log(format!("launch: {product} lancé via l'API du client"));
                        return Ok(());
                    }
                    Err(e) if e.detail.contains("423") => {
                        log(format!("launch: {product} déjà en cours de lancement (423)"));
                        in_progress = true;
                    }
                    Err(e) => last = e.detail,
                }
            }
        }
        tokio::time::sleep(Duration::from_millis(300)).await;
    }
    log(format!(
        "launch: API indisponible ({last}), repli sur la ligne de commande"
    ));
    let mut cmd = Command::new(exe);
    if let Some(dir) = exe.parent() {
        cmd.current_dir(dir);
    }
    cmd.arg(format!("--launch-product={product}"));
    cmd.arg("--launch-patchline=live");
    if let Some(l) = locale {
        cmd.arg(format!("--locale={l}"));
    }
    cmd.spawn()
        .map(|_| ())
        .map_err(|e| AppError::new("launch_failed", e.to_string()))
}
