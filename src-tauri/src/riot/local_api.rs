//! Client des API locales prévues pour les clients Riot (Riot Client et LCU), en 127.0.0.1.
//!
//! Le certificat de ces API est auto-signé : on l'accepte, mais ce client n'est utilisé que pour
//! `https://127.0.0.1:<port>` (la vraie barrière est le mot de passe du lockfile, lisible
//! uniquement par l'utilisateur de la session). Le mot de passe n'est jamais journalisé.

use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use reqwest::Method;
use serde_json::Value;
use std::path::Path;
use std::time::Duration;

use crate::error::{AppError, AppResult};

#[derive(Clone)]
pub struct Lockfile {
    pub port: u16,
    pub password: String,
}

/// Format : `nom:pid:port:motdepasse:protocole`.
pub fn read_lockfile(path: &Path) -> Option<Lockfile> {
    // Le fichier est verrouillé en écriture par le client : lecture partagée uniquement.
    let text = std::fs::read_to_string(path).ok()?;
    let parts: Vec<&str> = text.trim().split(':').collect();
    if parts.len() < 5 {
        return None;
    }
    Some(Lockfile {
        port: parts[2].parse().ok()?,
        password: parts[3].to_string(),
    })
}

#[derive(Clone)]
pub struct LocalApi {
    client: reqwest::Client,
    base: String,
    auth: String,
}

impl LocalApi {
    pub fn new(lf: &Lockfile) -> AppResult<Self> {
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .no_proxy()
            .timeout(Duration::from_secs(6))
            .build()?;
        Ok(Self {
            client,
            base: format!("https://127.0.0.1:{}", lf.port),
            auth: format!("Basic {}", B64.encode(format!("riot:{}", lf.password))),
        })
    }

    pub fn from_lockfile(path: &Path) -> Option<Self> {
        read_lockfile(path).and_then(|lf| Self::new(&lf).ok())
    }

    pub async fn request(
        &self,
        method: Method,
        path: &str,
        body: Option<&Value>,
    ) -> AppResult<Value> {
        let mut req = self
            .client
            .request(method, format!("{}{}", self.base, path))
            .header("Authorization", &self.auth)
            .header("Accept", "application/json");
        if let Some(b) = body {
            req = req.json(b);
        }
        let resp = req.send().await?;
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(AppError::new(
                "local_api",
                format!("{} {}", status.as_u16(), path),
            ));
        }
        if text.trim().is_empty() {
            return Ok(Value::Null);
        }
        Ok(serde_json::from_str(&text).unwrap_or(Value::String(text)))
    }

    /// Code HTTP d'une requête GET (None si le client ne répond pas du tout).
    pub async fn status(&self, path: &str) -> Option<u16> {
        self.client
            .get(format!("{}{}", self.base, path))
            .header("Authorization", &self.auth)
            .send()
            .await
            .ok()
            .map(|r| r.status().as_u16())
    }

    pub async fn get(&self, path: &str) -> AppResult<Value> {
        self.request(Method::GET, path, None).await
    }
    pub async fn post(&self, path: &str, body: &Value) -> AppResult<Value> {
        self.request(Method::POST, path, Some(body)).await
    }
    pub async fn put(&self, path: &str, body: &Value) -> AppResult<Value> {
        self.request(Method::PUT, path, Some(body)).await
    }
    pub async fn patch(&self, path: &str, body: &Value) -> AppResult<Value> {
        self.request(Method::PATCH, path, Some(body)).await
    }
    pub async fn delete(&self, path: &str) -> AppResult<Value> {
        self.request(Method::DELETE, path, None).await
    }
}
