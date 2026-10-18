//! Petits utilitaires partagés : horodatage, écriture atomique, journal sans secret.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// Millisecondes depuis l'époque Unix.
pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// Écrit un fichier de façon atomique : fichier temporaire + fsync + renommage.
/// Si `keep_backup`, l'ancienne version est conservée en `.bak`.
pub fn atomic_write(path: &Path, data: &[u8], keep_backup: bool) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension(format!(
        "{}.tmp",
        path.extension().and_then(|e| e.to_str()).unwrap_or("")
    ));
    {
        let mut f = fs::File::create(&tmp)?;
        f.write_all(data)?;
        f.sync_all()?;
    }
    if keep_backup && path.exists() {
        let bak = backup_path(path);
        let _ = fs::copy(path, bak);
    }
    fs::rename(&tmp, path)
}

pub fn backup_path(path: &Path) -> PathBuf {
    let mut s = path.as_os_str().to_owned();
    s.push(".bak");
    PathBuf::from(s)
}

// ── Journal ────────────────────────────────────────────────────────────────
// Ne reçoit JAMAIS de secret : uniquement des étapes, des statuts et des chemins.

static LOG_PATH: OnceLock<Mutex<Option<PathBuf>>> = OnceLock::new();

pub fn init_log(path: PathBuf) {
    // On garde le journal court : au-delà de 512 Ko on repart de zéro.
    if fs::metadata(&path)
        .map(|m| m.len() > 512 * 1024)
        .unwrap_or(false)
    {
        let _ = fs::remove_file(&path);
    }
    *LOG_PATH.get_or_init(|| Mutex::new(None)).lock().unwrap() = Some(path);
}

pub fn log(msg: impl AsRef<str>) {
    let line = format!("{} {}\n", now_ms(), msg.as_ref());
    #[cfg(debug_assertions)]
    eprint!("{line}");
    if let Some(lock) = LOG_PATH.get() {
        if let Ok(guard) = lock.lock() {
            if let Some(p) = guard.as_ref() {
                if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(p) {
                    let _ = f.write_all(line.as_bytes());
                }
            }
        }
    }
}

/// Masque un email : `prince@gmail.com` → `pr***@gm***.com`.
pub fn mask_email(email: &str) -> String {
    let email = email.trim();
    let Some((user, domain)) = email.split_once('@') else {
        return mask_part(email);
    };
    let (host, tld) = match domain.rsplit_once('.') {
        Some((h, t)) => (h, format!(".{t}")),
        None => (domain, String::new()),
    };
    format!("{}@{}{}", mask_part(user), mask_part(host), tld)
}

fn mask_part(s: &str) -> String {
    let head: String = s.chars().take(2).collect();
    format!("{head}***")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn masks_email() {
        assert_eq!(mask_email("prince@gmail.com"), "pr***@gm***.com");
        assert_eq!(mask_email("a@b"), "a***@b***");
        assert_eq!(mask_email("noat"), "no***");
    }

    #[test]
    fn atomic_write_keeps_backup() {
        let dir = std::env::temp_dir().join(format!("rs-test-{}", uuid::Uuid::new_v4()));
        let p = dir.join("f.json");
        atomic_write(&p, b"one", true).unwrap();
        atomic_write(&p, b"two", true).unwrap();
        assert_eq!(fs::read(&p).unwrap(), b"two");
        assert_eq!(fs::read(backup_path(&p)).unwrap(), b"one");
        let _ = fs::remove_dir_all(dir);
    }
}
