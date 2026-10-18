//! Emplacements connus du Riot Client et des jeux (Windows et macOS).

use std::path::{Path, PathBuf};

use crate::paths::{home, local_app_data};

fn riot_support_dir() -> PathBuf {
    if cfg!(target_os = "macos") {
        home()
            .join("Library")
            .join("Application Support")
            .join("Riot Games")
            .join("Riot Client")
    } else {
        local_app_data().join("Riot Games").join("Riot Client")
    }
}

/// Dossier Data du Riot Client : contient la session « Rester connecté ».
pub fn riot_client_data() -> PathBuf {
    riot_support_dir().join("Data")
}

pub fn riot_client_lockfile() -> PathBuf {
    riot_support_dir().join("Config").join("lockfile")
}

/// Fichiers de session à capturer / restaurer. Les variantes historiques sont gardées par prudence.
pub const SESSION_FILES: [&str; 3] = [
    "RiotGamesPrivateSettings.yaml",
    "RiotPrivateSettings.yaml",
    "RiotClientPrivateSettings.yaml",
];

fn shared_dir() -> PathBuf {
    if cfg!(target_os = "macos") {
        PathBuf::from("/Users/Shared/Riot Games")
    } else {
        std::env::var_os("ProgramData")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(r"C:\ProgramData"))
            .join("Riot Games")
    }
}

/// Chemin de RiotClientServices, lu dans le manifeste officiel, avec repli sur l'emplacement par défaut.
pub fn riot_client_services(custom: &str) -> Option<PathBuf> {
    if !custom.trim().is_empty() && Path::new(custom).exists() {
        return Some(PathBuf::from(custom));
    }
    let manifest = shared_dir().join("RiotClientInstalls.json");
    if let Ok(bytes) = std::fs::read(&manifest) {
        if let Ok(v) = serde_json::from_slice::<serde_json::Value>(&bytes) {
            for key in ["rc_default", "rc_live"] {
                if let Some(p) = v.get(key).and_then(|x| x.as_str()) {
                    if Path::new(p).exists() {
                        return Some(PathBuf::from(p));
                    }
                }
            }
        }
    }
    let defaults: &[&str] = if cfg!(target_os = "macos") {
        &["/Users/Shared/Riot Games/Riot Client.app/Contents/MacOS/RiotClientServices"]
    } else {
        &[r"C:\Riot Games\Riot Client\RiotClientServices.exe"]
    };
    defaults.iter().map(PathBuf::from).find(|p| p.exists())
}

/// Identifiant produit Riot pour un jeu de l'app.
pub fn product_id(game: &str) -> &'static str {
    match game {
        "val" => "valorant",
        "tft" => "teamfighttactics",
        _ => "league_of_legends",
    }
}

/// Fichier de réglages produit (contient la langue choisie et le dossier d'installation).
pub fn product_settings(game: &str) -> PathBuf {
    let p = product_id(game);
    shared_dir()
        .join("Metadata")
        .join(format!("{p}.live"))
        .join(format!("{p}.live.product_settings.yaml"))
}

/// Dossier d'installation d'un jeu, lu dans les réglages produit.
pub fn install_dir(game: &str) -> Option<PathBuf> {
    let text = std::fs::read_to_string(product_settings(game)).ok()?;
    let re = regex::Regex::new(r#"(?m)^\s*product_install_full_path:\s*"([^"]+)""#).ok()?;
    let path = PathBuf::from(re.captures(&text)?.get(1)?.as_str());
    path.exists().then_some(path)
}

pub fn lol_install(custom: &str) -> Option<PathBuf> {
    if !custom.trim().is_empty() && Path::new(custom).exists() {
        return Some(PathBuf::from(custom));
    }
    if let Some(p) = install_dir("lol") {
        return Some(p);
    }
    let candidates: &[&str] = if cfg!(target_os = "macos") {
        &["/Applications/League of Legends.app/Contents/LoL"]
    } else {
        &[
            r"C:\Riot Games\League of Legends",
            r"D:\Riot Games\League of Legends",
        ]
    };
    candidates.iter().map(PathBuf::from).find(|p| p.exists())
}

pub fn lcu_lockfile(custom: &str) -> Option<PathBuf> {
    lol_install(custom).map(|p| p.join("lockfile"))
}

pub fn lol_config_dir(custom: &str) -> Option<PathBuf> {
    lol_install(custom).map(|p| p.join("Config"))
}

/// Config Valorant locale : un dossier par compte, nommé `{puuid}-{shard}`.
pub fn valorant_config_root() -> PathBuf {
    local_app_data()
        .join("VALORANT")
        .join("Saved")
        .join("Config")
}

/// Change la langue d'un jeu avant lancement (réglage produit + fichier du client LoL).
pub fn set_game_locale(game: &str, locale: &str, lol_custom: &str) {
    let re = regex::Regex::new(r#"(?m)^(\s*locale:\s*)"[^"]*""#).expect("regex");
    let set = |path: PathBuf| {
        if let Ok(text) = std::fs::read_to_string(&path) {
            let updated = re
                .replace_all(&text, format!(r#"${{1}}"{locale}""#).as_str())
                .to_string();
            if updated != text {
                let _ = crate::util::atomic_write(&path, updated.as_bytes(), false);
            }
        }
    };
    set(product_settings(game));
    if game != "val" {
        if let Some(cfg) = lol_config_dir(lol_custom) {
            set(cfg.join("LeagueClientSettings.yaml"));
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn locale_regex_only_touches_locale_key() {
        let re = regex::Regex::new(r#"(?m)^(\s*locale:\s*)"[^"]*""#).unwrap();
        let text =
            "locale_data:\n    default_locale: \"en_GB\"\nsettings:\n    locale: \"fr_FR\"\n";
        let out = re.replace_all(text, r#"${1}"ja_JP""#);
        assert!(out.contains("default_locale: \"en_GB\""));
        assert!(out.contains("    locale: \"ja_JP\""));
    }
}
