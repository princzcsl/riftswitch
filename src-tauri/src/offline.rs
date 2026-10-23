//! Mode hors ligne : proxy XMPP local, sur le principe de Deceive (molenzwiebel/deceive).
//!
//! 1. Le Riot Client est lancé avec `--client-config-url=http://127.0.0.1:<port>` : notre proxy de
//!    configuration relaie les requêtes vers les serveurs Riot et remplace l'hôte du chat par
//!    `riftswitch.duckdns.org`, qui pointe vers 127.0.0.1.
//! 2. Le client se connecte alors à notre proxy de chat. Le Riot Client refusant les certificats
//!    maison, le proxy présente le certificat public de ce domaine (renouvelé chaque mois par le
//!    workflow certificate.yml), puis relaie vers le vrai serveur en réécrivant les présences.
//!
//! Aucun message automatique n'est injecté dans le chat. Rien ne touche au processus du jeu.

use regex::Regex;
use std::io::Read;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;

use crate::error::{AppError, AppResult};
use crate::util::log;
use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};

/// Pointe vers 127.0.0.1 ; certificat public publié sur la pré-version proxy-cert.
const PROXY_DOMAIN: &str = "riftswitch.duckdns.org";
const CERT_URL: &str =
    "https://github.com/princzcsl/riftswitch/releases/download/proxy-cert/riftswitch-localhost";

const CONFIG_UPSTREAM: &str = "https://clientconfig.rpg.riotgames.com";
const PAS_URL: &str = "https://riot-geo.pas.si.riotgames.com/pas/v1/service/chat";

pub struct OfflineProxy {
    pub config_port: u16,
    pub chat_port: u16,
    status: AtomicU8,
    chat_target: Mutex<Option<(String, u16)>>,
    last_presence: Mutex<Option<String>>,
    injectors: Mutex<Vec<mpsc::UnboundedSender<Vec<u8>>>>,
}

impl OfflineProxy {
    pub async fn start(offline: bool, cache_dir: &std::path::Path) -> AppResult<Arc<Self>> {
        let (certs, key) = proxy_certificate(cache_dir).await?;
        let chat_listener = TcpListener::bind("127.0.0.1:0").await?;
        let chat_port = chat_listener.local_addr()?.port();
        let http = tiny_http::Server::http("127.0.0.1:0")
            .map_err(|e| AppError::new("offline_proxy", e.to_string()))?;
        let config_port = http.server_addr().to_ip().map(|a| a.port()).unwrap_or(0);

        let proxy = Arc::new(Self {
            config_port,
            chat_port,
            status: AtomicU8::new(ChatStatus::from_offline(offline) as u8),
            chat_target: Mutex::new(None),
            last_presence: Mutex::new(None),
            injectors: Mutex::new(Vec::new()),
        });

        // Proxy de configuration (thread dédié, très peu de requêtes).
        let p = proxy.clone();
        std::thread::spawn(move || {
            for req in http.incoming_requests() {
                let p = p.clone();
                std::thread::spawn(move || p.handle_config(req));
            }
        });

        // Proxy de chat.
        let tls = server_tls(certs, key)?;
        let p = proxy.clone();
        tauri::async_runtime::spawn(async move {
            loop {
                let Ok((sock, _)) = chat_listener.accept().await else {
                    continue;
                };
                let (p, tls) = (p.clone(), tls.clone());
                tauri::async_runtime::spawn(async move {
                    if let Err(e) = p.handle_chat(sock, tls).await {
                        log(format!("offline: connexion chat terminée ({}: {})", e.code, e.detail));
                    }
                });
            }
        });
        log(format!(
            "offline: proxy démarré config={config_port} chat={chat_port}"
        ));
        Ok(proxy)
    }

    pub fn launch_args(&self) -> Vec<String> {
        vec![format!(
            "--client-config-url=http://127.0.0.1:{}",
            self.config_port
        )]
    }

    pub fn status(&self) -> ChatStatus {
        ChatStatus::from_u8(self.status.load(Ordering::Relaxed))
    }

    pub fn is_offline(&self) -> bool {
        self.status() != ChatStatus::Online
    }

    pub fn set_offline(&self, offline: bool) {
        self.set_status(ChatStatus::from_offline(offline));
    }

    /// Change le statut à chaud et renvoie la dernière présence pour l'appliquer tout de suite.
    pub fn set_status(&self, status: ChatStatus) {
        self.status.store(status as u8, Ordering::Relaxed);
        let last = self.last_presence.lock().unwrap().clone();
        if let Some(raw) = last {
            let out = rewrite_presences(&raw, status);
            self.injectors
                .lock()
                .unwrap()
                .retain(|tx| tx.send(out.clone().into_bytes()).is_ok());
        }
    }

    // ── Proxy de configuration ──────────────────────────────────────────

    fn handle_config(&self, mut req: tiny_http::Request) {
        let url = req.url().to_string();
        let method = req.method().as_str().to_string();
        let mut body = Vec::new();
        let _ = req.as_reader().take(1024 * 1024).read_to_end(&mut body);
        let forward: Vec<(String, String)> = req
            .headers()
            .iter()
            .filter(|h| {
                let f = h.field.as_str().as_str().to_ascii_lowercase();
                matches!(
                    f.as_str(),
                    "user-agent"
                        | "x-riot-entitlements-jwt"
                        | "authorization"
                        | "accept"
                        | "x-riot-rso-identity-jwt"
                )
            })
            .map(|h| {
                (
                    h.field.as_str().as_str().to_string(),
                    h.value.as_str().to_string(),
                )
            })
            .collect();
        let auth = forward
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case("authorization"))
            .map(|(_, v)| v.clone());

        let result = tauri::async_runtime::block_on(async {
            let client = reqwest::Client::builder().build()?;
            let m = reqwest::Method::from_bytes(method.as_bytes()).unwrap_or(reqwest::Method::GET);
            let mut r = client.request(m, format!("{CONFIG_UPSTREAM}{url}"));
            for (k, v) in &forward {
                r = r.header(k, v);
            }
            if !body.is_empty() {
                r = r.body(body.clone());
            }
            let resp = r.send().await?;
            let status = resp.status().as_u16();
            let text = resp.text().await?;
            let text = if url.starts_with("/api/v1/config/player") && status == 200 {
                self.patch_player_config(&text, auth.as_deref())
                    .await
                    .unwrap_or(text)
            } else {
                text
            };
            Ok::<_, reqwest::Error>((status, text))
        });

        let response = match result {
            Ok((status, text)) => tiny_http::Response::from_string(text)
                .with_status_code(status)
                .with_header(
                    tiny_http::Header::from_bytes("Content-Type", "application/json").unwrap(),
                ),
            Err(_) => tiny_http::Response::from_string("{}").with_status_code(502),
        };
        let _ = req.respond(response);
    }

    async fn patch_player_config(&self, text: &str, auth: Option<&str>) -> Option<String> {
        let mut v: serde_json::Value = serde_json::from_str(text).ok()?;
        let obj = v.as_object_mut()?;
        let mut host = obj
            .get("chat.host")
            .and_then(|x| x.as_str())
            .map(str::to_string);
        let port = obj
            .get("chat.port")
            .and_then(|x| x.as_u64())
            .unwrap_or(5223) as u16;

        if let Some(aff) = obj
            .get("chat.affinities")
            .and_then(|x| x.as_object())
            .cloned()
        {
            let enabled = obj
                .get("chat.affinity.enabled")
                .and_then(|x| x.as_bool())
                .unwrap_or(false);
            if enabled {
                if let Some(a) = fetch_affinity(auth).await {
                    if let Some(h) = aff.get(&a).and_then(|x| x.as_str()) {
                        host = Some(h.to_string());
                    }
                }
            }
            let local: serde_json::Map<String, serde_json::Value> = aff
                .keys()
                .map(|k| (k.clone(), serde_json::Value::String(PROXY_DOMAIN.into())))
                .collect();
            obj.insert("chat.affinities".into(), serde_json::Value::Object(local));
        }
        if let Some(h) = host {
            *self.chat_target.lock().unwrap() = Some((h, port));
            obj.insert("chat.host".into(), PROXY_DOMAIN.into());
            obj.insert("chat.port".into(), self.chat_port.into());
            obj.insert("chat.allow_bad_cert.enabled".into(), true.into());
            log("offline: configuration du chat redirigée vers le proxy local");
        }
        serde_json::to_string(&v).ok()
    }

    // ── Proxy de chat ───────────────────────────────────────────────────

    async fn handle_chat(
        self: Arc<Self>,
        sock: TcpStream,
        tls: Arc<rustls::ServerConfig>,
    ) -> AppResult<()> {
        let acceptor = tokio_rustls::TlsAcceptor::from(tls);
        let client = acceptor
            .accept(sock)
            .await
            .map_err(|e| AppError::new("offline_accept", e.to_string()))?;

        let target = { self.chat_target.lock().unwrap().clone() };
        let (host, port) = target.ok_or_else(|| AppError::code("offline_no_target"))?;
        let upstream_tcp = TcpStream::connect((host.as_str(), port))
            .await
            .map_err(|e| AppError::new("offline_upstream", format!("{host}:{port} {e}")))?;
        let connector = tokio_rustls::TlsConnector::from(client_tls());
        let name = rustls::pki_types::ServerName::try_from(host.clone())
            .map_err(|_| AppError::code("offline_proxy"))?;
        let upstream = connector
            .connect(name, upstream_tcp)
            .await
            .map_err(|e| AppError::new("offline_upstream", format!("{host} {e}")))?;

        let (mut c_read, mut c_write) = tokio::io::split(client);
        let (mut s_read, mut s_write) = tokio::io::split(upstream);
        let (tx, mut rx) = mpsc::unbounded_channel::<Vec<u8>>();
        self.injectors.lock().unwrap().push(tx.clone());

        // Écrivain vers le serveur : données du client (réécrites) + injections (changement de statut).
        let writer = tauri::async_runtime::spawn(async move {
            while let Some(buf) = rx.recv().await {
                if s_write.write_all(&buf).await.is_err() {
                    break;
                }
            }
        });

        // Serveur → client : relais tel quel.
        let down = tauri::async_runtime::spawn(async move {
            let mut buf = vec![0u8; 16 * 1024];
            loop {
                match s_read.read(&mut buf).await {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        if c_write.write_all(&buf[..n]).await.is_err() {
                            break;
                        }
                    }
                }
            }
        });

        // Client → serveur : on tamponne jusqu'à avoir des présences complètes.
        let mut pending: Vec<u8> = Vec::new();
        let mut buf = vec![0u8; 16 * 1024];
        loop {
            let n = match c_read.read(&mut buf).await {
                Ok(0) | Err(_) => break,
                Ok(n) => n,
            };
            pending.extend_from_slice(&buf[..n]);
            let Ok(text) = std::str::from_utf8(&pending) else {
                if pending.len() > 256 * 1024 {
                    let _ = tx.send(std::mem::take(&mut pending));
                }
                continue;
            };
            if text.contains("<presence") {
                if !presence_complete(text) && pending.len() < 256 * 1024 {
                    continue;
                }
                let raw = text.to_string();
                if let Some(stanza) = broadcast_presence(&raw) {
                    *self.last_presence.lock().unwrap() = Some(stanza);
                }
                let out = rewrite_presences(&raw, self.status());
                let _ = tx.send(out.into_bytes());
            } else {
                let _ = tx.send(pending.clone());
            }
            pending.clear();
        }
        drop(tx);
        down.abort();
        writer.abort();
        Ok(())
    }
}

async fn fetch_affinity(auth: Option<&str>) -> Option<String> {
    let auth = auth?;
    let jwt = reqwest::Client::new()
        .get(PAS_URL)
        .header("Authorization", auth)
        .send()
        .await
        .ok()?
        .text()
        .await
        .ok()?;
    let payload = jwt.split('.').nth(1)?;
    use base64::Engine;
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(payload.trim_end_matches('='))
        .ok()?;
    let v: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    v.get("affinity")
        .and_then(|x| x.as_str())
        .map(str::to_string)
}

fn provider() -> Arc<rustls::crypto::CryptoProvider> {
    static P: OnceLock<Arc<rustls::crypto::CryptoProvider>> = OnceLock::new();
    P.get_or_init(|| Arc::new(rustls::crypto::ring::default_provider()))
        .clone()
}

/// Certificat du domaine, en cache local rafraîchi chaque semaine. Le cache sert de secours
/// si GitHub est injoignable.
async fn proxy_certificate(
    cache_dir: &std::path::Path,
) -> AppResult<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>)> {
    let dir = cache_dir.join("proxy-cert");
    let (crt_path, key_path) = (dir.join("localhost.crt"), dir.join("localhost.key"));
    let fresh = std::fs::metadata(&crt_path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.elapsed().ok())
        .is_some_and(|age| age < std::time::Duration::from_secs(7 * 86_400));
    if !fresh {
        match download_certificate().await {
            Ok((crt, key)) => {
                let _ = std::fs::create_dir_all(&dir);
                let _ = std::fs::write(&crt_path, crt);
                let _ = std::fs::write(&key_path, key);
                log("offline: certificat du proxy téléchargé");
            }
            Err(e) => log(format!("offline: certificat non téléchargé ({}), cache utilisé", e.detail)),
        }
    }
    let crt = std::fs::read(&crt_path).map_err(|_| AppError::code("offline_cert"))?;
    let key = std::fs::read(&key_path).map_err(|_| AppError::code("offline_cert"))?;
    let certs = CertificateDer::pem_slice_iter(&crt)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::new("offline_cert", e.to_string()))?;
    let key = PrivateKeyDer::from_pem_slice(&key).map_err(|e| AppError::new("offline_cert", e.to_string()))?;
    Ok((certs, key))
}

async fn download_certificate() -> AppResult<(Vec<u8>, Vec<u8>)> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()?;
    let get = |ext: &'static str| {
        let client = client.clone();
        async move {
            let r = client.get(format!("{CERT_URL}.{ext}")).send().await?.error_for_status()?;
            Ok::<_, AppError>(r.bytes().await?.to_vec())
        }
    };
    Ok((get("crt").await?, get("key").await?))
}

fn server_tls(
    certs: Vec<CertificateDer<'static>>,
    key: PrivateKeyDer<'static>,
) -> AppResult<Arc<rustls::ServerConfig>> {
    let cfg = rustls::ServerConfig::builder_with_provider(provider())
        .with_safe_default_protocol_versions()
        .map_err(|e| AppError::new("offline_proxy", e.to_string()))?
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|e| AppError::new("offline_proxy", e.to_string()))?;
    Ok(Arc::new(cfg))
}

fn client_tls() -> Arc<rustls::ClientConfig> {
    // Magasin Windows + Mozilla, comme Deceive (.NET valide avec le magasin Windows).
    let mut roots = rustls::RootCertStore::empty();
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    for cert in rustls_native_certs::load_native_certs().certs {
        let _ = roots.add(cert);
    }
    let cfg = rustls::ClientConfig::builder_with_provider(provider())
        .with_safe_default_protocol_versions()
        .expect("versions TLS")
        .with_root_certificates(roots)
        .with_no_client_auth();
    Arc::new(cfg)
}

fn presence_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?s)<presence(?P<attrs>[^>]*?)>(?P<body>.*?)</presence>").expect("regex")
    })
}

/// Les présences ouvertes ont-elles toutes leur balise fermante ?
fn presence_complete(text: &str) -> bool {
    let Some(last) = text.rfind("<presence") else {
        return true;
    };
    let tail = &text[last..];
    match tail.find('>') {
        Some(gt) if tail[..gt].ends_with('/') => true,
        Some(_) => tail.contains("</presence>"),
        None => false,
    }
}

/// Dernière présence diffusée (sans destinataire) du texte, à renvoyer lors d'un changement de statut.
fn broadcast_presence(text: &str) -> Option<String> {
    presence_re()
        .captures_iter(text)
        .filter(|c| !c["attrs"].contains("to="))
        .last()
        .map(|c| c[0].to_string())
}

/// Réécrit les présences diffusées (sans destinataire) pour apparaître hors ligne.
/// Les présences adressées (salons de partie, lobby) restent intactes pour que le jeu fonctionne.
/// Statut affiché aux amis. Absent garde le jeu en cours visible : « absent » pour VALORANT
/// et le Riot Client, « occupé » pour League of Legends (qui n'a pas d'absent choisi).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatStatus {
    Online = 0,
    Away = 1,
    Mobile = 3,
    Offline = 4,
}

impl ChatStatus {
    pub fn from_offline(offline: bool) -> Self {
        if offline {
            Self::Offline
        } else {
            Self::Online
        }
    }
    fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::Away,
            3 => Self::Mobile,
            4 => Self::Offline,
            _ => Self::Online,
        }
    }
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "online" => Self::Online,
            "away" | "busy" => Self::Away,
            "mobile" => Self::Mobile,
            "offline" => Self::Offline,
            _ => return None,
        })
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Online => "online",
            Self::Away => "away",
            Self::Mobile => "mobile",
            Self::Offline => "offline",
        }
    }
    /// Valeur XMPP de <show>.
    fn show(self) -> &'static str {
        match self {
            Self::Online => "chat",
            Self::Away => "away",
            Self::Mobile => "mobile",
            Self::Offline => "offline",
        }
    }
}

pub fn rewrite_presences(text: &str, status: ChatStatus) -> String {
    if status == ChatStatus::Online {
        return text.to_string();
    }
    let hide_game = matches!(status, ChatStatus::Mobile | ChatStatus::Offline);
    static SHOW: OnceLock<Regex> = OnceLock::new();
    static STATUS: OnceLock<Regex> = OnceLock::new();
    static GAMES: OnceLock<Regex> = OnceLock::new();
    static ST: OnceLock<Regex> = OnceLock::new();
    let show = SHOW.get_or_init(|| Regex::new(r"(?s)<show>.*?</show>").unwrap());
    let status_re = STATUS.get_or_init(|| Regex::new(r"(?s)<status>.*?</status>|<status/>").unwrap());
    let games = GAMES.get_or_init(|| {
        Regex::new(r"(?s)<(league_of_legends|valorant|bacon|lion|riot_client)>.*?</(league_of_legends|valorant|bacon|lion|riot_client)>").unwrap()
    });
    let st = ST.get_or_init(|| Regex::new(r"(?s)<st>.*?</st>").unwrap());
    static LOL: OnceLock<Regex> = OnceLock::new();
    let lol = LOL.get_or_init(|| Regex::new(r"(?s)<league_of_legends>.*?</league_of_legends>").unwrap());

    presence_re()
        .replace_all(text, |c: &regex::Captures| {
            let attrs = &c["attrs"];
            if attrs.contains("to=") {
                return c[0].to_string();
            }
            let tag = format!("<show>{}</show>", status.show());
            let mut body = c["body"].to_string();
            body = if show.is_match(&body) {
                show.replace_all(&body, tag.as_str()).to_string()
            } else {
                format!("{tag}{body}")
            };
            if hide_game {
                body = status_re.replace_all(&body, "").to_string();
                body = games.replace_all(&body, "").to_string();
            }
            body = st
                .replace_all(&body, format!("<st>{}</st>", status.show()).as_str())
                .to_string();
            if status == ChatStatus::Away {
                body = lol
                    .replace_all(&body, |m: &regex::Captures| st.replace_all(&m[0], "<st>dnd</st>").to_string())
                    .to_string();
            }
            format!("<presence{attrs}>{body}</presence>")
        })
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewrites_broadcast_only() {
        let input = r#"<presence id="1"><show>chat</show><status>hi</status><games><league_of_legends><st>dnd</st><p>x</p></league_of_legends><keystone><st>chat</st></keystone></games></presence><presence to="room@lobby"><show>chat</show></presence>"#;
        let out = rewrite_presences(input, ChatStatus::Offline);
        assert!(out.contains(r#"<presence id="1"><show>offline</show><games><keystone><st>offline</st></keystone></games></presence>"#));
        assert!(out.contains(r#"<presence to="room@lobby"><show>chat</show></presence>"#));
        assert!(!out.contains("league_of_legends"));
    }

    #[test]
    fn away_keeps_the_game_and_is_busy_on_league() {
        let input = r#"<presence id="1"><show>chat</show><games><league_of_legends><st>chat</st><p>x</p></league_of_legends><valorant><st>chat</st></valorant></games></presence>"#;
        let away = rewrite_presences(input, ChatStatus::Away);
        assert!(away.contains("<show>away</show>"));
        assert!(away.contains("<league_of_legends><st>dnd</st>"));
        assert!(away.contains("<valorant><st>away</st></valorant>"));
        let mobile = rewrite_presences(input, ChatStatus::Mobile);
        assert!(mobile.contains("<show>mobile</show>") && !mobile.contains("league_of_legends"));
        assert_eq!(rewrite_presences(input, ChatStatus::Online), input);
    }

    #[test]
    fn detects_incomplete_presence() {
        assert!(!presence_complete("<presence id='1'><show>chat"));
        assert!(presence_complete(
            "<presence id='1'><show>chat</show></presence>"
        ));
        assert!(presence_complete("<presence/>"));
        assert!(presence_complete("<iq/>"));
    }
}
