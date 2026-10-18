//! Emplacement des données de RiftSwitch.
//!
//! - Installée : `%APPDATA%\RiftSwitch` (Windows) / `~/Library/Application Support/RiftSwitch` (macOS).
//! - Portable : si un fichier `portable.flag` est à côté de l'exécutable, tout va dans `./data`
//!   et le mot de passe maître devient obligatoire (pas de DPAPI / Trousseau).
//! - Développement : la variable `RIFTSWITCH_DATA_DIR` force un dossier isolé.

use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Paths {
    pub root: PathBuf,
    pub portable: bool,
}

impl Paths {
    pub fn resolve() -> Self {
        if let Ok(dir) = std::env::var("RIFTSWITCH_DATA_DIR") {
            if !dir.trim().is_empty() {
                return Self {
                    root: PathBuf::from(dir),
                    portable: false,
                };
            }
        }
        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                if dir.join("portable.flag").exists() {
                    return Self {
                        root: dir.join("data"),
                        portable: true,
                    };
                }
            }
        }
        let base = if cfg!(target_os = "macos") {
            home().join("Library").join("Application Support")
        } else {
            std::env::var_os("APPDATA")
                .map(PathBuf::from)
                .unwrap_or_else(|| home().join("AppData").join("Roaming"))
        };
        Self {
            root: base.join("RiftSwitch"),
            portable: false,
        }
    }

    pub fn vault(&self) -> PathBuf {
        self.root.join("vault.rsv")
    }
    pub fn settings(&self) -> PathBuf {
        self.root.join("settings.json")
    }
    pub fn profiles_dir(&self) -> PathBuf {
        self.root.join("profiles")
    }
    pub fn cache_dir(&self) -> PathBuf {
        self.root.join("cache")
    }
    pub fn log(&self) -> PathBuf {
        self.root.join("riftswitch.log")
    }
}

pub fn home() -> PathBuf {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn local_app_data() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join("AppData").join("Local"))
}
