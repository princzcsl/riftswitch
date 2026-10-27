//! Profils de configuration de jeu (touches, interface, son, graphismes) pour LoL et Valorant.
//!
//! Pas de secret dans un profil : stockés en clair dans `profiles/<id>.json`, exportables en
//! fichier ou en code court (`RSC1.` + JSON compressé en base64url).

pub mod ini;

use base64::engine::general_purpose::URL_SAFE_NO_PAD as B64URL;
use base64::Engine;
use flate2::read::DeflateDecoder;
use flate2::write::DeflateEncoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::error::{AppError, AppResult};
use crate::riot::{local_api::LocalApi, paths};
use crate::util::{self, log, now_ms};
use ini::IniDoc;

pub const LOL_INPUT_SECTIONS: [&str; 4] = ["GameEvents", "HUDEvents", "Quickbinds", "ShopEvents"];
/// Clés de [General] rangées ailleurs que dans « Interface ».
const LOL_GENERAL_GRAPHICS: [&str; 6] = [
    "Width",
    "Height",
    "WindowMode",
    "WaitForVerticalSync",
    "ShowGodray",
    "HideEyeCandy",
];
const LOL_GENERAL_SOUND: [&str; 5] = [
    "EnableAudio",
    "ThemeMusic",
    "EnableCustomAnnouncer",
    "EnableCosmeticArenaAudioTFT",
    "EnableAllCosmeticArenaAudioTFT",
];

/// Catégorie d'une clé de game.cfg (None : jamais copiée, ex. réglages mobiles).
pub fn lol_cat(section: &str, key: &str) -> Option<&'static str> {
    match section {
        "Volume" | "Voice" => Some("sound"),
        "Performance" => Some("graphics"),
        "General" if LOL_GENERAL_GRAPHICS.contains(&key) => Some("graphics"),
        "General" if LOL_GENERAL_SOUND.contains(&key) => Some("sound"),
        "Mobile" | "GameModes" | "Replay" => None,
        _ => Some("interface"),
    }
}

/// Clés propres à un PC ou à une version du jeu : jamais affichées ni copiées d'un compte à l'autre.
pub fn is_volatile(key: &str) -> bool {
    let k = key.rsplit("::").next().unwrap_or(key);
    const PARTS: [&str; 10] = [
        "DeviceID",
        "MonitorIndex",
        "Benchmark",
        "LastConfirmed",
        "LastUserConfirmed",
        "LastRecommended",
        "WindowPos",
        "PreferredMonitor",
        "LastSeen",
        "Tutorial",
    ];
    // États « déjà vu / déjà accepté » et stand de tir : propres au compte, sans intérêt à copier.
    const PREFIXES: [&str; 6] = [
        "HasSeen",
        "HasAccepted",
        "HasEver",
        "LastAccepted",
        "ShootingRange",
        "ContextAware",
    ];
    k.ends_with("Version")
        || PARTS.iter().any(|p| k.contains(p))
        || PREFIXES.iter().any(|p| k.starts_with(p))
}

/// Réglage VALORANT rangé dans « Son ».
pub fn is_val_sound(key: &str) -> bool {
    let k = key.rsplit("::").next().unwrap_or(key);
    [
        "Volume",
        "Music",
        "Sound",
        "Audio",
        "VoiceChat",
        "Speaker",
        "Microphone",
        "Mic",
    ]
    .iter()
    .any(|p| k.contains(p))
}

/// Catégorie d'une clé de RiotUserSettings (VALORANT).
pub fn val_cat(key: &str) -> &'static str {
    if key.rsplit("::").next().unwrap_or(key).contains("Crosshair") {
        "crosshair"
    } else if is_val_sound(key) {
        "sound"
    } else {
        "interface"
    }
}

/// Anciennes catégories VALORANT : « settings » couvrait interface, viseur et son.
pub fn has_cat(categories: &[String], cat: &str) -> bool {
    categories.iter().any(|c| {
        c == cat || (c == "settings" && matches!(cat, "interface" | "sound" | "crosshair"))
    })
}

const VAL_KEYBINDS: &str = "WindowsClient/BackupKeybinds.json";
const VAL_SETTINGS: &str = "Windows/RiotUserSettings.ini";
const VAL_GRAPHICS: &str = "WindowsClient/GameUserSettings.ini";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LolConfig {
    /// input.ini : touches.
    pub input: IniDoc,
    /// game.cfg : interface, son, graphismes.
    pub game: IniDoc,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ValConfig {
    /// Fichiers bruts par chemin relatif au dossier du compte.
    pub files: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ConfigProfile {
    pub id: String,
    pub name: String,
    /// "lol" | "val"
    pub game: String,
    pub created_at: i64,
    pub updated_at: i64,
    /// "client" | "files" | "import"
    pub source: String,
    /// Catégories contenues et appliquées : keybinds, interface, sound, graphics (LoL) ;
    /// keybinds, settings, graphics (Valorant).
    pub categories: Vec<String>,
    pub notes: String,
    /// Compte d'origine : identifiant local (vide pour un profil importé), Riot ID et icône.
    pub source_account: String,
    pub source_name: String,
    pub source_icon: Option<i64>,
    pub lol: Option<LolConfig>,
    pub val: Option<ValConfig>,
}

pub struct ProfileStore {
    dir: PathBuf,
}

impl ProfileStore {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    fn path(&self, id: &str) -> AppResult<PathBuf> {
        if id.is_empty() || !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return Err(AppError::code("profile_not_found"));
        }
        Ok(self.dir.join(format!("{id}.json")))
    }

    pub fn list(&self) -> Vec<ConfigProfile> {
        let mut out: Vec<ConfigProfile> = std::fs::read_dir(&self.dir)
            .into_iter()
            .flatten()
            .flatten()
            .filter(|e| e.path().extension().is_some_and(|x| x == "json"))
            .filter_map(|e| std::fs::read(e.path()).ok())
            .filter_map(|b| serde_json::from_slice(&b).ok())
            .collect();
        out.sort_by_key(|p| std::cmp::Reverse(p.updated_at));
        out
    }

    pub fn get(&self, id: &str) -> AppResult<ConfigProfile> {
        let bytes =
            std::fs::read(self.path(id)?).map_err(|_| AppError::code("profile_not_found"))?;
        Ok(serde_json::from_slice(&bytes)?)
    }

    pub fn save(&self, mut p: ConfigProfile) -> AppResult<ConfigProfile> {
        if p.id.is_empty() {
            p.id = uuid::Uuid::new_v4().to_string();
            p.created_at = now_ms();
        }
        if !matches!(p.game.as_str(), "lol" | "val") {
            return Err(AppError::new("invalid", "game"));
        }
        p.updated_at = now_ms();
        util::atomic_write(&self.path(&p.id)?, &serde_json::to_vec_pretty(&p)?, false)?;
        Ok(p)
    }

    pub fn delete(&self, id: &str) -> AppResult<()> {
        let p = self.path(id)?;
        if p.exists() {
            std::fs::remove_file(p)?;
        }
        Ok(())
    }
}

// ── Codes de partage ────────────────────────────────────────────────────────

pub fn to_code(p: &ConfigProfile) -> AppResult<String> {
    let mut clean = p.clone();
    clean.id.clear();
    clean.notes.clear();
    clean.source_account.clear();
    let json = serde_json::to_vec(&clean)?;
    let mut enc = DeflateEncoder::new(Vec::new(), Compression::best());
    enc.write_all(&json)?;
    Ok(format!("RSC1.{}", B64URL.encode(enc.finish()?)))
}

pub fn from_code(code: &str) -> AppResult<ConfigProfile> {
    let body = code
        .trim()
        .strip_prefix("RSC1.")
        .ok_or_else(|| AppError::code("code_invalid"))?;
    let bytes = B64URL
        .decode(body.trim())
        .map_err(|_| AppError::code("code_invalid"))?;
    // Limite de taille décompressée : un code piégé ne doit pas saturer la mémoire.
    let mut out = Vec::new();
    DeflateDecoder::new(&bytes[..])
        .take(4 * 1024 * 1024)
        .read_to_end(&mut out)
        .map_err(|_| AppError::code("code_invalid"))?;
    let mut p: ConfigProfile =
        serde_json::from_slice(&out).map_err(|_| AppError::code("code_invalid"))?;
    p.id.clear();
    p.source_account.clear();
    p.source = "import".into();
    Ok(p)
}

// ── League of Legends ───────────────────────────────────────────────────────

/// Capture la config LoL : via le client si connecté (réglages du compte), sinon via les fichiers.
pub async fn capture_lol(
    lcu: Option<&LocalApi>,
    lol_custom: &str,
) -> AppResult<(LolConfig, &'static str)> {
    if let Some(api) = lcu {
        if let (Ok(input), Ok(game)) = (
            api.get("/lol-game-settings/v1/input-settings").await,
            api.get("/lol-game-settings/v1/game-settings").await,
        ) {
            if input.is_object() {
                return Ok((
                    LolConfig {
                        input: IniDoc::from_json(&input),
                        game: IniDoc::from_json(&game),
                    },
                    "client",
                ));
            }
        }
    }
    let dir = paths::lol_config_dir(lol_custom).ok_or_else(|| AppError::code("lol_not_found"))?;
    let input = std::fs::read_to_string(dir.join("input.ini")).unwrap_or_default();
    let game = std::fs::read_to_string(dir.join("game.cfg")).unwrap_or_default();
    if input.is_empty() && game.is_empty() {
        return Err(AppError::code("config_not_found"));
    }
    Ok((
        LolConfig {
            input: IniDoc::parse(&input),
            game: IniDoc::parse(&game),
        },
        "files",
    ))
}

/// Applique un profil LoL. Via le client (synchronisé sur le compte connecté) si possible,
/// et toujours dans les fichiers locaux.
pub async fn apply_lol(
    p: &ConfigProfile,
    lcu: Option<&LocalApi>,
    lol_custom: &str,
) -> AppResult<Vec<String>> {
    let cfg = p
        .lol
        .as_ref()
        .ok_or_else(|| AppError::new("invalid", "lol"))?;
    let keybinds = has_cat(&p.categories, "keybinds");
    let game = cfg.game.filtered(|s, k| {
        !is_volatile(k) && lol_cat(s, k).is_some_and(|c| has_cat(&p.categories, c))
    });
    let mut done = Vec::new();

    if let Some(dir) = paths::lol_config_dir(lol_custom) {
        if keybinds {
            let path = dir.join("input.ini");
            let mut current = IniDoc::parse(&std::fs::read_to_string(&path).unwrap_or_default());
            current.merge_sections(&cfg.input, &LOL_INPUT_SECTIONS);
            util::atomic_write(&path, current.to_text().as_bytes(), true)?;
        }
        if !game.sections.is_empty() {
            let path = dir.join("game.cfg");
            let mut current = IniDoc::parse(&std::fs::read_to_string(&path).unwrap_or_default());
            current.merge_sections(&game, &game.names());
            util::atomic_write(&path, current.to_text().as_bytes(), true)?;
        }
        done.push("files".to_string());
    }

    if let Some(api) = lcu {
        let mut ok = true;
        if keybinds {
            let current = api
                .get("/lol-game-settings/v1/input-settings")
                .await
                .unwrap_or(Value::Null);
            let body = cfg.input.only(&LOL_INPUT_SECTIONS).to_json_like(&current);
            ok &= api
                .patch("/lol-game-settings/v1/input-settings", &body)
                .await
                .is_ok();
        }
        if !game.sections.is_empty() {
            let current = api
                .get("/lol-game-settings/v1/game-settings")
                .await
                .unwrap_or(Value::Null);
            let body = game.to_json_like(&current);
            ok &= api
                .patch("/lol-game-settings/v1/game-settings", &body)
                .await
                .is_ok();
        }
        let _ = api.post("/lol-game-settings/v1/save", &Value::Null).await;
        if ok {
            done.push("client".to_string());
        }
        log(format!("configs: profil LoL appliqué via client ok={ok}"));
    }
    if done.is_empty() {
        return Err(AppError::code("lol_not_found"));
    }
    Ok(done)
}

// ── Valorant ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValDir {
    pub name: String,
    pub puuid: String,
    pub shard: String,
    pub modified: i64,
    pub has_keybinds: bool,
}

pub fn list_val_dirs() -> Vec<ValDir> {
    let re = regex::Regex::new(
        r"^([0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})-([a-z]+)$",
    )
    .expect("regex");
    let root = paths::valorant_config_root();
    let mut out: Vec<ValDir> = std::fs::read_dir(&root)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            let caps = re.captures(&name)?;
            let modified = e
                .metadata()
                .and_then(|m| m.modified())
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_millis() as i64)
                .unwrap_or(0);
            Some(ValDir {
                puuid: caps[1].to_string(),
                shard: caps[2].to_string(),
                has_keybinds: e.path().join(VAL_KEYBINDS).exists(),
                name,
                modified,
            })
        })
        .collect();
    out.sort_by_key(|d| std::cmp::Reverse(d.modified));
    out
}

fn val_dir(name: &str) -> AppResult<PathBuf> {
    if !list_val_dirs().iter().any(|d| d.name == name) {
        return Err(AppError::code("val_dir_not_found"));
    }
    Ok(paths::valorant_config_root().join(name))
}

pub fn capture_val(dir_name: &str) -> AppResult<ValConfig> {
    let dir = val_dir(dir_name)?;
    let mut files = BTreeMap::new();
    for rel in [VAL_KEYBINDS, VAL_SETTINGS, VAL_GRAPHICS] {
        if let Ok(text) = std::fs::read_to_string(dir.join(rel)) {
            files.insert(rel.to_string(), text);
        }
    }
    if files.is_empty() {
        return Err(AppError::code("config_not_found"));
    }
    Ok(ValConfig { files })
}

pub fn apply_val(p: &ConfigProfile, dir_name: &str) -> AppResult<Vec<String>> {
    let cfg = p
        .val
        .as_ref()
        .ok_or_else(|| AppError::new("invalid", "val"))?;
    let dir = val_dir(dir_name)?;
    let cats = &p.categories;
    let mut done = Vec::new();
    if has_cat(cats, "keybinds") {
        if let Some(text) = cfg.files.get(VAL_KEYBINDS) {
            util::atomic_write(&dir.join(VAL_KEYBINDS), text.as_bytes(), true)?;
            done.push("keybinds".to_string());
        }
    }
    // Réglages : fusion clé par clé dans le fichier du compte, sans les clés propres au PC.
    let settings = ["interface", "sound", "crosshair"]
        .iter()
        .any(|c| has_cat(cats, c));
    let graphics = has_cat(cats, "graphics");
    // Fichier, et catégorie d'une clé : interface / son pour RiotUserSettings, graphismes sinon.
    let parts = [(VAL_SETTINGS, settings), (VAL_GRAPHICS, graphics)];
    let keep = |rel: &str, k: &str| rel == VAL_GRAPHICS || has_cat(cats, val_cat(k));
    for (rel, wanted) in parts {
        let Some(text) = cfg.files.get(rel).filter(|_| wanted) else {
            continue;
        };
        let src = IniDoc::parse(text).filtered(|_, k| !is_volatile(k) && keep(rel, k));
        let path = dir.join(rel);
        let mut current = IniDoc::parse(&std::fs::read_to_string(&path).unwrap_or_default());
        current.merge_sections(&src, &src.names());
        util::atomic_write(&path, current.to_text().as_bytes(), true)?;
        done.push(rel.to_string());
    }
    Ok(done)
}

pub fn val_dir_for(puuid: Option<&str>) -> Option<String> {
    let puuid = puuid?;
    list_val_dirs()
        .into_iter()
        .find(|d| d.puuid == puuid)
        .map(|d| d.name)
}

pub fn read_file(path: &Path) -> AppResult<ConfigProfile> {
    let bytes = std::fs::read(path)?;
    let mut p: ConfigProfile =
        serde_json::from_slice(&bytes).map_err(|_| AppError::code("import_invalid"))?;
    p.id.clear();
    p.source_account.clear();
    p.source = "import".into();
    Ok(p)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_roundtrip() {
        let p = ConfigProfile {
            id: "abc".into(),
            name: "Main keys".into(),
            game: "lol".into(),
            categories: vec!["keybinds".into()],
            lol: Some(LolConfig {
                input: IniDoc::parse("[GameEvents]\nevtCastSpell1=[q]\n"),
                game: IniDoc::default(),
            }),
            ..Default::default()
        };
        let code = to_code(&p).unwrap();
        assert!(code.starts_with("RSC1."));
        let back = from_code(&code).unwrap();
        assert_eq!(back.name, "Main keys");
        assert!(back.id.is_empty());
        assert_eq!(back.lol.unwrap().input, p.lol.unwrap().input);
        assert!(from_code("RSC1.@@@").is_err());
        assert!(from_code("nope").is_err());
    }

    #[test]
    fn volatile_and_sound_keys() {
        assert!(is_volatile("DefaultMonitorDeviceID"));
        assert!(is_volatile("EAresIntSettingName::LocalSettingsVersion"));
        assert!(is_volatile("CfgVersion"));
        assert!(!is_volatile("ResolutionSizeX"));
        assert!(is_val_sound("EAresFloatSettingName::OverallVolume"));
        assert!(!is_val_sound("EAresFloatSettingName::MouseSensitivity"));
        assert!(has_cat(&["settings".into()], "sound"));
        assert!(is_volatile("EAresBoolSettingName::HasSeenBTEModal"));
        assert_eq!(
            val_cat("EAresStringSettingName::SavedCrosshairProfileData"),
            "crosshair"
        );
        assert_eq!(lol_cat("General", "GameMouseSpeed"), Some("interface"));
        assert_eq!(lol_cat("General", "WindowMode"), Some("graphics"));
        assert_eq!(lol_cat("Mobile", "AppRegion"), None);
        assert!(!has_cat(&["settings".into()], "graphics"));
    }

    #[test]
    fn store_rejects_path_traversal() {
        let store = ProfileStore::new(std::env::temp_dir());
        assert!(store.get("../secret").is_err());
    }
}
