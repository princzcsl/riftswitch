//! Coffre-fort chiffré de RiftSwitch.
//!
//! Format sur disque (`vault.rsv`, JSON) :
//! - une clé de coffre (DEK) aléatoire de 256 bits chiffre la charge utile (XChaCha20-Poly1305) ;
//! - mode `password` : la DEK est enveloppée par une clé dérivée du mot de passe maître (Argon2id) ;
//! - mode `os` : la DEK est confiée au Gestionnaire d'identifiants Windows (DPAPI) / Trousseau macOS ;
//! - l'en-tête (mode, paramètres KDF) est authentifié via l'AAD : impossible de le modifier sans casser le déchiffrement.
//!
//! Le mot de passe maître n'est jamais stocké. Les secrets de chaque compte sont scellés une seconde
//! fois (AAD = id du compte) et ne sont déchiffrés qu'au moment de la bascule.

pub mod crypto;
pub mod model;

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use zeroize::Zeroizing;

use crate::error::{AppError, AppResult};
use crate::util;
use crypto::{KdfParams, Key, Sealed};
use model::{Account, AccountSecrets, VaultData};

const FORMAT: &str = "riftswitch-vault";
const FILE_VERSION: u32 = 1;
pub const DATA_VERSION: u32 = 1;
const KEYRING_SERVICE: &str = "RiftSwitch";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultFile {
    pub format: String,
    pub version: u32,
    /// "password" | "os" | "export"
    pub mode: String,
    pub kdf: Option<KdfParams>,
    pub wrapped_key: Option<Sealed>,
    pub keyring_account: Option<String>,
    pub payload: Sealed,
}

impl VaultFile {
    fn header_aad(version: u32, mode: &str, kdf: Option<&KdfParams>) -> Vec<u8> {
        let kdf = kdf.map(|k| k.aad()).unwrap_or_else(|| "-".into());
        format!("{FORMAT}:v{version}:{mode}:{kdf}").into_bytes()
    }

    fn aad(&self) -> Vec<u8> {
        Self::header_aad(self.version, &self.mode, self.kdf.as_ref())
    }

    fn read(path: &Path) -> AppResult<Self> {
        let bytes = std::fs::read(path).map_err(|_| AppError::code("vault_missing"))?;
        let file: VaultFile =
            serde_json::from_slice(&bytes).map_err(|_| AppError::new("vault_corrupt", "json"))?;
        if file.format != FORMAT || file.version != FILE_VERSION {
            return Err(AppError::new("vault_corrupt", "format"));
        }
        Ok(file)
    }
}

/// Protection de la clé de coffre.
#[derive(Clone)]
enum Protection {
    Password { kdf: KdfParams, wrapped: Sealed },
    Os { keyring_account: String },
}

/// Coffre déverrouillé. Le déposer (drop) efface la clé de la mémoire.
pub struct Vault {
    key: Key,
    protection: Protection,
    path: PathBuf,
    pub data: VaultData,
}

impl Vault {
    pub fn exists(path: &Path) -> bool {
        path.exists()
    }

    /// Mode du coffre existant sans le déverrouiller ("password" | "os").
    pub fn peek_mode(path: &Path) -> Option<String> {
        VaultFile::read(path).ok().map(|f| f.mode)
    }

    /// Crée un nouveau coffre vide. `password = None` → clé confiée au système (DPAPI / Trousseau).
    pub fn create(path: &Path, password: Option<&str>) -> AppResult<Self> {
        let key = crypto::random_key();
        let protection = Self::protect(&key, password)?;
        let vault = Self {
            key,
            protection,
            path: path.to_path_buf(),
            data: VaultData {
                version: DATA_VERSION,
                ..Default::default()
            },
        };
        vault.save()?;
        Ok(vault)
    }

    fn protect(key: &Key, password: Option<&str>) -> AppResult<Protection> {
        match password {
            Some(pw) => {
                if pw.chars().count() < 8 {
                    return Err(AppError::code("password_too_short"));
                }
                let kdf = crypto::calibrate(500);
                let kek = crypto::derive_key(pw.as_bytes(), &kdf)?;
                let wrapped = crypto::seal(&kek, key.as_ref(), &wrapped_aad(&kdf));
                Ok(Protection::Password { kdf, wrapped })
            }
            None => {
                let account = format!("vault-key-{}", uuid::Uuid::new_v4());
                os_key_store(&account, key)?;
                Ok(Protection::Os {
                    keyring_account: account,
                })
            }
        }
    }

    pub fn unlock(path: &Path, password: Option<&str>) -> AppResult<Self> {
        let file = VaultFile::read(path)?;
        let (key, protection) = match file.mode.as_str() {
            "password" => {
                let pw = password.ok_or_else(|| AppError::code("password_required"))?;
                let kdf = file
                    .kdf
                    .clone()
                    .ok_or_else(|| AppError::new("vault_corrupt", "kdf"))?;
                let wrapped = file
                    .wrapped_key
                    .clone()
                    .ok_or_else(|| AppError::new("vault_corrupt", "key"))?;
                let kek = crypto::derive_key(pw.as_bytes(), &kdf)?;
                let raw = crypto::open(&kek, &wrapped, &wrapped_aad(&kdf))
                    .map_err(|_| AppError::code("wrong_password"))?;
                (to_key(&raw)?, Protection::Password { kdf, wrapped })
            }
            "os" => {
                let account = file
                    .keyring_account
                    .clone()
                    .ok_or_else(|| AppError::new("vault_corrupt", "keyring"))?;
                (
                    os_key_load(&account)?,
                    Protection::Os {
                        keyring_account: account,
                    },
                )
            }
            _ => return Err(AppError::new("vault_corrupt", "mode")),
        };
        let plain = crypto::open(&key, &file.payload, &file.aad())
            .map_err(|_| AppError::code("vault_tampered"))?;
        let mut data: VaultData = serde_json::from_slice(&plain)
            .map_err(|_| AppError::new("vault_corrupt", "payload"))?;
        if data.version < DATA_VERSION {
            // Sauvegarde chiffrée (copie brute du fichier) avant toute migration de schéma.
            let _ = std::fs::copy(path, path.with_extension(format!("v{}.bak", data.version)));
            data.version = DATA_VERSION;
        }
        let vault = Self {
            key,
            protection,
            path: path.to_path_buf(),
            data,
        };
        Ok(vault)
    }

    pub fn mode(&self) -> &'static str {
        match self.protection {
            Protection::Password { .. } => "password",
            Protection::Os { .. } => "os",
        }
    }

    pub fn save(&self) -> AppResult<()> {
        let json = Zeroizing::new(serde_json::to_vec(&self.data)?);
        let (mode, kdf, wrapped, keyring_account) = match &self.protection {
            Protection::Password { kdf, wrapped } => {
                ("password", Some(kdf.clone()), Some(wrapped.clone()), None)
            }
            Protection::Os { keyring_account } => ("os", None, None, Some(keyring_account.clone())),
        };
        let aad = VaultFile::header_aad(FILE_VERSION, mode, kdf.as_ref());
        let payload = crypto::seal(&self.key, &json, &aad);
        let file = VaultFile {
            format: FORMAT.into(),
            version: FILE_VERSION,
            mode: mode.into(),
            kdf,
            wrapped_key: wrapped,
            keyring_account,
            payload,
        };
        util::atomic_write(&self.path, &serde_json::to_vec_pretty(&file)?, true)?;
        Ok(())
    }

    /// Change le mot de passe maître, ou bascule entre mode mot de passe et mode système.
    pub fn change_protection(&mut self, new_password: Option<&str>) -> AppResult<()> {
        let old = self.protection.clone();
        self.protection = Self::protect(&self.key, new_password)?;
        self.save()?;
        if let Protection::Os { keyring_account } = old {
            os_key_delete(&keyring_account);
        }
        Ok(())
    }

    // ── Secrets par compte ───────────────────────────────────────────────

    pub fn seal_secrets(&self, account: &Account, secrets: &AccountSecrets) -> AppResult<Sealed> {
        let json = Zeroizing::new(serde_json::to_vec(secrets)?);
        Ok(crypto::seal(&self.key, &json, &account.secret_aad()))
    }

    pub fn open_secrets(&self, account: &Account) -> AppResult<AccountSecrets> {
        let Some(sealed) = account.secret.as_ref() else {
            return Ok(AccountSecrets::default());
        };
        let plain = crypto::open(&self.key, sealed, &account.secret_aad())?;
        serde_json::from_slice(&plain).map_err(|_| AppError::new("vault_corrupt", "secret"))
    }

    /// Modifie les secrets d'un compte (déchiffrement, modification, rescellement, effacement).
    pub fn update_secrets(
        &mut self,
        account_id: &str,
        f: impl FnOnce(&mut AccountSecrets),
    ) -> AppResult<()> {
        let idx = self.index_of(account_id)?;
        let mut secrets = self.open_secrets(&self.data.accounts[idx])?;
        f(&mut secrets);
        let sealed = self.seal_secrets(&self.data.accounts[idx], &secrets)?;
        self.data.accounts[idx].secret = Some(sealed);
        Ok(())
    }

    pub fn index_of(&self, account_id: &str) -> AppResult<usize> {
        self.data
            .accounts
            .iter()
            .position(|a| a.id == account_id)
            .ok_or_else(|| AppError::code("account_not_found"))
    }

    // ── Export / import chiffrés (phrase de passe dédiée) ─────────────────

    pub fn export(&self, passphrase: &str) -> AppResult<Vec<u8>> {
        if passphrase.chars().count() < 8 {
            return Err(AppError::code("password_too_short"));
        }
        let export_key = crypto::random_key();
        let mut data = self.data.clone();
        for acc in data.accounts.iter_mut() {
            if acc.secret.is_some() {
                let secrets = self.open_secrets(acc)?;
                acc.secret = Some(crypto::seal(
                    &export_key,
                    &Zeroizing::new(serde_json::to_vec(&secrets)?),
                    &acc.secret_aad(),
                ));
            }
        }
        let kdf = crypto::calibrate(500);
        let kek = crypto::derive_key(passphrase.as_bytes(), &kdf)?;
        let wrapped = crypto::seal(&kek, export_key.as_ref(), &wrapped_aad(&kdf));
        let aad = VaultFile::header_aad(FILE_VERSION, "export", Some(&kdf));
        let payload = crypto::seal(
            &export_key,
            &Zeroizing::new(serde_json::to_vec(&data)?),
            &aad,
        );
        let file = VaultFile {
            format: FORMAT.into(),
            version: FILE_VERSION,
            mode: "export".into(),
            kdf: Some(kdf),
            wrapped_key: Some(wrapped),
            keyring_account: None,
            payload,
        };
        Ok(serde_json::to_vec_pretty(&file)?)
    }

    /// Importe un export : ajoute les comptes et dossiers absents. Renvoie le nombre de comptes ajoutés.
    pub fn import(&mut self, bytes: &[u8], passphrase: &str) -> AppResult<usize> {
        let file: VaultFile =
            serde_json::from_slice(bytes).map_err(|_| AppError::code("import_invalid"))?;
        if file.format != FORMAT || file.mode != "export" {
            return Err(AppError::code("import_invalid"));
        }
        let kdf = file
            .kdf
            .clone()
            .ok_or_else(|| AppError::code("import_invalid"))?;
        let wrapped = file
            .wrapped_key
            .clone()
            .ok_or_else(|| AppError::code("import_invalid"))?;
        let kek = crypto::derive_key(passphrase.as_bytes(), &kdf)?;
        let raw = crypto::open(&kek, &wrapped, &wrapped_aad(&kdf))
            .map_err(|_| AppError::code("wrong_password"))?;
        let export_key = to_key(&raw)?;
        let plain = crypto::open(&export_key, &file.payload, &file.aad())?;
        let incoming: VaultData =
            serde_json::from_slice(&plain).map_err(|_| AppError::code("import_invalid"))?;

        let mut added = 0;
        for folder in incoming.folders {
            if !self.data.folders.iter().any(|f| f.id == folder.id) {
                self.data.folders.push(folder);
            }
        }
        for mut acc in incoming.accounts {
            if self.data.accounts.iter().any(|a| a.id == acc.id) {
                continue;
            }
            if let Some(sealed) = acc.secret.take() {
                let secret_plain = crypto::open(&export_key, &sealed, &acc.secret_aad())?;
                acc.secret = Some(crypto::seal(&self.key, &secret_plain, &acc.secret_aad()));
            }
            self.data.accounts.push(acc);
            added += 1;
        }
        for page in incoming.rune_library {
            if !self.data.rune_library.iter().any(|p| p.id == page.id) {
                self.data.rune_library.push(page);
            }
        }
        self.save()?;
        Ok(added)
    }

    /// Supprime définitivement le coffre, sa sauvegarde et la clé système associée.
    pub fn destroy(path: &Path) {
        if let Ok(file) = VaultFile::read(path) {
            if let Some(acc) = file.keyring_account {
                os_key_delete(&acc);
            }
        }
        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_file(util::backup_path(path));
    }
}

fn wrapped_aad(kdf: &KdfParams) -> Vec<u8> {
    format!("{FORMAT}:dek:{}", kdf.aad()).into_bytes()
}

fn to_key(raw: &[u8]) -> AppResult<Key> {
    if raw.len() != crypto::KEY_LEN {
        return Err(AppError::new("vault_corrupt", "key length"));
    }
    let mut k = Zeroizing::new([0u8; crypto::KEY_LEN]);
    k.copy_from_slice(raw);
    Ok(k)
}

// ── Clé protégée par le système (DPAPI via le Gestionnaire d'identifiants / Trousseau) ──

fn os_key_store(account: &str, key: &Key) -> AppResult<()> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, account)
        .map_err(|e| AppError::new("os_keystore", e.to_string()))?;
    entry
        .set_secret(key.as_ref())
        .map_err(|e| AppError::new("os_keystore", e.to_string()))
}

fn os_key_load(account: &str) -> AppResult<Key> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, account)
        .map_err(|e| AppError::new("os_keystore", e.to_string()))?;
    let secret = Zeroizing::new(
        entry
            .get_secret()
            .map_err(|e| AppError::new("os_keystore", e.to_string()))?,
    );
    to_key(&secret)
}

fn os_key_delete(account: &str) {
    if let Ok(entry) = keyring::Entry::new(KEYRING_SERVICE, account) {
        let _ = entry.delete_credential();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_vault() -> PathBuf {
        std::env::temp_dir()
            .join(format!("rs-vault-{}", uuid::Uuid::new_v4()))
            .join("vault.rsv")
    }

    #[test]
    fn create_unlock_roundtrip() {
        let p = tmp_vault();
        let mut v = Vault::create(&p, Some("correct horse")).unwrap();
        v.data.accounts.push(Account {
            id: "a1".into(),
            label: "Main".into(),
            ..Default::default()
        });
        v.save().unwrap();
        drop(v);
        let v = Vault::unlock(&p, Some("correct horse")).unwrap();
        assert_eq!(v.data.accounts[0].label, "Main");
        assert_eq!(v.mode(), "password");
    }

    #[test]
    fn wrong_password_is_rejected() {
        let p = tmp_vault();
        Vault::create(&p, Some("correct horse")).unwrap();
        let err = Vault::unlock(&p, Some("wrong horse!")).err().unwrap();
        assert_eq!(err.code, "wrong_password");
    }

    #[test]
    fn short_password_is_rejected() {
        assert_eq!(
            Vault::create(&tmp_vault(), Some("short"))
                .err()
                .unwrap()
                .code,
            "password_too_short"
        );
    }

    #[test]
    fn nothing_in_clear_on_disk() {
        let p = tmp_vault();
        let mut v = Vault::create(&p, Some("correct horse")).unwrap();
        let mut acc = Account {
            id: "a1".into(),
            email: "visible@example.com".into(),
            ..Default::default()
        };
        let secrets = AccountSecrets {
            username: Some("loginname".into()),
            password: Some("SuperSecretPw!".into()),
            refresh_token: Some("RefreshTokenSecret".into()),
            id_token: Some("IdTokenSecret".into()),
        };
        acc.secret = Some(v.seal_secrets(&acc, &secrets).unwrap());
        v.data.accounts.push(acc);
        v.save().unwrap();
        let raw = String::from_utf8_lossy(&std::fs::read(&p).unwrap()).to_string();
        for needle in [
            "visible@example.com",
            "loginname",
            "SuperSecretPw!",
            "RefreshTokenSecret",
            "IdTokenSecret",
            "correct horse",
        ] {
            assert!(!raw.contains(needle), "{needle} trouvé en clair");
        }
    }

    #[test]
    fn header_tampering_is_detected() {
        let p = tmp_vault();
        Vault::create(&p, Some("correct horse")).unwrap();
        let mut file: VaultFile = serde_json::from_slice(&std::fs::read(&p).unwrap()).unwrap();
        // Affaiblir les paramètres KDF casse l'enveloppe de clé → refus.
        file.kdf.as_mut().unwrap().t += 1;
        std::fs::write(&p, serde_json::to_vec(&file).unwrap()).unwrap();
        assert!(Vault::unlock(&p, Some("correct horse")).is_err());
    }

    #[test]
    fn payload_tampering_is_detected() {
        let p = tmp_vault();
        Vault::create(&p, Some("correct horse")).unwrap();
        let mut file: VaultFile = serde_json::from_slice(&std::fs::read(&p).unwrap()).unwrap();
        file.payload.ct[3] ^= 0x40;
        std::fs::write(&p, serde_json::to_vec(&file).unwrap()).unwrap();
        assert_eq!(
            Vault::unlock(&p, Some("correct horse")).err().unwrap().code,
            "vault_tampered"
        );
    }

    #[test]
    fn secrets_are_bound_to_their_account() {
        let p = tmp_vault();
        let v = Vault::create(&p, Some("correct horse")).unwrap();
        let a = Account {
            id: "a".into(),
            ..Default::default()
        };
        let mut b = Account {
            id: "b".into(),
            ..Default::default()
        };
        let sealed = v
            .seal_secrets(&a, &AccountSecrets::with_password("pw"))
            .unwrap();
        // Copier le secret de A sur B ne doit pas fonctionner (AAD = id du compte).
        b.secret = Some(sealed);
        assert!(v.open_secrets(&b).is_err());
    }

    #[test]
    fn export_import_roundtrip() {
        let mut src = Vault::create(&tmp_vault(), Some("correct horse")).unwrap();
        let mut acc = Account {
            id: "x".into(),
            label: "Smurf".into(),
            ..Default::default()
        };
        acc.secret = Some(
            src.seal_secrets(&acc, &AccountSecrets::with_password("pw123456"))
                .unwrap(),
        );
        src.data.accounts.push(acc);
        let blob = src.export("export phrase").unwrap();
        assert!(!String::from_utf8_lossy(&blob).contains("pw123456"));

        let mut dst = Vault::create(&tmp_vault(), Some("another pass")).unwrap();
        assert!(dst.import(&blob, "bad phrase!!").is_err());
        assert_eq!(dst.import(&blob, "export phrase").unwrap(), 1);
        let s = dst.open_secrets(&dst.data.accounts[0]).unwrap();
        assert_eq!(s.password.as_deref(), Some("pw123456"));
        // Réimport : pas de doublon.
        assert_eq!(dst.import(&blob, "export phrase").unwrap(), 0);
    }

    #[test]
    fn change_password() {
        let p = tmp_vault();
        let mut v = Vault::create(&p, Some("correct horse")).unwrap();
        v.change_protection(Some("brand new pass")).unwrap();
        drop(v);
        assert!(Vault::unlock(&p, Some("correct horse")).is_err());
        assert!(Vault::unlock(&p, Some("brand new pass")).is_ok());
    }
}
