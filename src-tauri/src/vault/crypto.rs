//! Primitives cryptographiques du coffre.
//!
//! - AEAD : XChaCha20-Poly1305 (nonce aléatoire de 192 bits, aucun risque de réutilisation).
//! - KDF : Argon2id, ≥ 64 Mio, itérations calibrées pour ~500 ms sur la machine.
//! - Toutes les clés sont dans des `Zeroizing` : effacées à la libération.

use argon2::{Algorithm, Argon2, Params, Version};
use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::time::Instant;
use zeroize::Zeroizing;

use crate::error::{AppError, AppResult};

pub const KEY_LEN: usize = 32;
pub type Key = Zeroizing<[u8; KEY_LEN]>;

pub fn random_bytes(len: usize) -> Vec<u8> {
    let mut buf = vec![0u8; len];
    getrandom::getrandom(&mut buf).expect("générateur aléatoire du système indisponible");
    buf
}

pub fn random_key() -> Key {
    let mut k = Zeroizing::new([0u8; KEY_LEN]);
    getrandom::getrandom(k.as_mut()).expect("générateur aléatoire du système indisponible");
    k
}

/// Données chiffrées + nonce. Sérialisé en base64.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sealed {
    #[serde(with = "b64")]
    pub nonce: Vec<u8>,
    #[serde(with = "b64")]
    pub ct: Vec<u8>,
}

pub fn seal(key: &[u8; KEY_LEN], plaintext: &[u8], aad: &[u8]) -> Sealed {
    let cipher = XChaCha20Poly1305::new(key.into());
    let nonce = random_bytes(24);
    let ct = cipher
        .encrypt(
            XNonce::from_slice(&nonce),
            Payload {
                msg: plaintext,
                aad,
            },
        )
        .expect("chiffrement AEAD");
    Sealed { nonce, ct }
}

pub fn open(key: &[u8; KEY_LEN], sealed: &Sealed, aad: &[u8]) -> AppResult<Zeroizing<Vec<u8>>> {
    if sealed.nonce.len() != 24 {
        return Err(AppError::new("vault_corrupt", "nonce"));
    }
    let cipher = XChaCha20Poly1305::new(key.into());
    cipher
        .decrypt(
            XNonce::from_slice(&sealed.nonce),
            Payload {
                msg: &sealed.ct,
                aad,
            },
        )
        .map(Zeroizing::new)
        // Mauvaise clé et données altérées sont indiscernables : c'est voulu.
        .map_err(|_| AppError::code("decrypt_failed"))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KdfParams {
    pub alg: String,
    pub m_kib: u32,
    pub t: u32,
    pub p: u32,
    #[serde(with = "b64")]
    pub salt: Vec<u8>,
}

pub const MIN_M_KIB: u32 = 64 * 1024;
const MAX_M_KIB: u32 = 1024 * 1024;
const MAX_T: u32 = 32;

impl KdfParams {
    pub fn new(m_kib: u32, t: u32) -> Self {
        Self {
            alg: "argon2id".into(),
            m_kib,
            t,
            p: 1,
            salt: random_bytes(16),
        }
    }

    /// Bornes anti-DoS : un fichier de coffre modifié ne doit pas pouvoir demander 50 Go de RAM.
    pub fn validate(&self) -> AppResult<()> {
        let ok = self.alg == "argon2id"
            && (MIN_M_KIB..=MAX_M_KIB).contains(&self.m_kib)
            && (1..=MAX_T).contains(&self.t)
            && (1..=4).contains(&self.p)
            && self.salt.len() >= 16;
        if ok {
            Ok(())
        } else {
            Err(AppError::new("vault_corrupt", "kdf params"))
        }
    }

    /// Représentation stable incluse dans l'AAD : toute modification des paramètres casse l'authentification.
    pub fn aad(&self) -> String {
        format!(
            "{}:{}:{}:{}:{}",
            self.alg,
            self.m_kib,
            self.t,
            self.p,
            B64.encode(&self.salt)
        )
    }
}

pub fn derive_key(password: &[u8], params: &KdfParams) -> AppResult<Key> {
    params.validate()?;
    let p = Params::new(params.m_kib, params.t, params.p, Some(KEY_LEN))
        .map_err(|e| AppError::new("kdf", e.to_string()))?;
    let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, p);
    let mut out = Zeroizing::new([0u8; KEY_LEN]);
    argon
        .hash_password_into(password, &params.salt, out.as_mut())
        .map_err(|e| AppError::new("kdf", e.to_string()))?;
    Ok(out)
}

/// Calibre le nombre d'itérations pour viser ~500 ms (64 Mio de mémoire, au moins 2 passes).
pub fn calibrate(target_ms: u128) -> KdfParams {
    let probe = KdfParams::new(MIN_M_KIB, 1);
    let start = Instant::now();
    let _ = derive_key(b"calibration", &probe);
    let one_pass = start.elapsed().as_millis().max(1);
    let t = (target_ms / one_pass).clamp(2, 10) as u32;
    KdfParams::new(MIN_M_KIB, t)
}

mod b64 {
    use super::*;
    pub fn serialize<S: Serializer>(v: &[u8], s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&B64.encode(v))
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
        let s = String::deserialize(d)?;
        B64.decode(s.as_bytes()).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seal_open_roundtrip() {
        let k = random_key();
        let s = seal(&k, b"secret", b"aad");
        assert_eq!(&**open(&k, &s, b"aad").unwrap(), b"secret");
    }

    #[test]
    fn wrong_aad_or_key_fails() {
        let k = random_key();
        let s = seal(&k, b"secret", b"aad");
        assert!(open(&k, &s, b"other").is_err());
        assert!(open(&random_key(), &s, b"aad").is_err());
    }

    #[test]
    fn tampering_is_detected() {
        let k = random_key();
        let mut s = seal(&k, b"secret", b"aad");
        s.ct[0] ^= 1;
        assert!(open(&k, &s, b"aad").is_err());
    }

    #[test]
    fn nonces_are_unique() {
        let k = random_key();
        assert_ne!(seal(&k, b"x", b"").nonce, seal(&k, b"x", b"").nonce);
    }

    #[test]
    fn kdf_is_deterministic_and_bounded() {
        let p = KdfParams::new(MIN_M_KIB, 1);
        assert_eq!(
            *derive_key(b"pw", &p).unwrap(),
            *derive_key(b"pw", &p).unwrap()
        );
        assert_ne!(
            *derive_key(b"pw", &p).unwrap(),
            *derive_key(b"pw2", &p).unwrap()
        );
        let mut bad = p.clone();
        bad.m_kib = 10_000_000;
        assert!(derive_key(b"pw", &bad).is_err());
    }
}
