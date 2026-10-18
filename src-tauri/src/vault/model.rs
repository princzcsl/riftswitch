//! Modèle de données du coffre (déchiffré en mémoire).
//!
//! Les métadonnées des comptes sont chiffrées avec le coffre ; les secrets (mot de passe Riot,
//! session « Rester connecté ») sont en plus scellés compte par compte et ne sont déchiffrés
//! qu'au moment de s'en servir.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use zeroize::{Zeroize, ZeroizeOnDrop};

use super::crypto::Sealed;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct VaultData {
    pub version: u32,
    pub accounts: Vec<Account>,
    pub folders: Vec<Folder>,
    /// Dernier compte connu comme actif dans le Riot Client (sert quand le client est fermé).
    pub last_active_account: Option<String>,
    pub rune_library: Vec<RunePage>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Folder {
    pub id: String,
    pub name: String,
    pub color: String,
    pub order: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Rank {
    /// IRON … CHALLENGER (LoL/TFT) ou IRON … RADIANT (Valorant). Vide = non classé.
    pub tier: String,
    pub division: String,
    pub lp: i32,
    pub wins: i32,
    pub losses: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Ranks {
    pub lol_solo: Option<Rank>,
    pub lol_flex: Option<Rank>,
    pub tft: Option<Rank>,
    pub val: Option<Rank>,
    pub updated_at: Option<i64>,
    /// Les rangs ont-ils été saisis à la main (non écrasés par la synchro) ?
    pub manual: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Decay {
    /// Jours restants avant decay (Diamant+), si connus.
    pub days_remaining: Option<i32>,
    pub banked_games: Option<i32>,
    pub updated_at: Option<i64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Health {
    /// Restriction / pénalité de file connue (texte court venant du client).
    pub penalties: Vec<String>,
    pub honor_level: Option<i32>,
    pub updated_at: Option<i64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Playtime {
    pub lol: u64,
    pub tft: u64,
    pub val: u64,
    /// Dernières sessions (secondes jouées par jour, clé = AAAA-MM-JJ) pour le « temps de jeu récent ».
    pub recent: BTreeMap<String, u64>,
    /// Lancements via l'app, par jeu.
    pub launches: BTreeMap<String, u64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct GameLangs {
    pub lol: Option<String>,
    pub tft: Option<String>,
    pub val: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ProfileLinks {
    pub lol: Option<String>,
    pub val: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Alias {
    pub riot_id: String,
    pub seen_at: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Account {
    pub id: String,
    /// Nom affiché choisi par l'utilisateur (ex. « Main EUW »).
    pub label: String,
    pub game_name: String,
    pub tag_line: String,
    pub puuid: Option<String>,
    /// Email de contact du compte (affiché masqué uniquement).
    pub email: String,
    pub region: String,
    pub color: String,
    /// `glyph:<id>`, `champ:<Clé>` ou `data:image/png;base64,…` (import recadré).
    pub icon: String,
    pub tags: Vec<String>,
    pub folder_id: Option<String>,
    pub favorite: bool,
    pub created_at: i64,
    pub last_used_at: Option<i64>,
    pub last_game: Option<String>,
    pub langs: GameLangs,
    pub two_fa: bool,
    pub notes: String,
    pub champion_pool: Vec<String>,
    pub agent_pool: Vec<String>,
    pub roles: Vec<String>,
    pub aliases: Vec<Alias>,
    pub ranks: Ranks,
    pub decay: Decay,
    pub health: Health,
    pub playtime: Playtime,
    pub profiles: ProfileLinks,
    pub session_captured_at: Option<i64>,
    /// Indicateurs (pour l'UI) de la présence de secrets, sans les déchiffrer.
    pub has_password: bool,
    pub has_username: bool,
    /// Une paire de jetons RSO RiftSwitch est enregistrée pour ce compte.
    pub has_tokens: bool,
    /// Infos LoL lues sur le client (icône d'invocateur, niveau).
    pub profile_icon_id: Option<i64>,
    pub level: Option<i64>,
    /// Secrets scellés (AAD = id du compte). Jamais envoyés au front.
    pub secret: Option<Sealed>,
}

impl Account {
    pub fn riot_id(&self) -> String {
        if self.game_name.is_empty() {
            String::new()
        } else if self.tag_line.is_empty() {
            self.game_name.clone()
        } else {
            format!("{}#{}", self.game_name, self.tag_line)
        }
    }

    pub fn secret_aad(&self) -> Vec<u8> {
        format!("riftswitch:secret:{}", self.id).into_bytes()
    }

    /// Mémorise un Riot ID observé et garde l'historique des anciens pseudos.
    pub fn observe_riot_id(&mut self, game_name: &str, tag_line: &str, now: i64) {
        if game_name.is_empty() {
            return;
        }
        let changed = !self.game_name.eq_ignore_ascii_case(game_name)
            || !self.tag_line.eq_ignore_ascii_case(tag_line);
        if changed && !self.game_name.is_empty() {
            let old = self.riot_id();
            if !self.aliases.iter().any(|a| a.riot_id == old) {
                self.aliases.push(Alias {
                    riot_id: old,
                    seen_at: now,
                });
            }
        }
        self.game_name = game_name.to_string();
        self.tag_line = tag_line.to_string();
    }

    pub fn matches_identity(&self, puuid: Option<&str>, game_name: &str, tag_line: &str) -> bool {
        if let (Some(a), Some(b)) = (self.puuid.as_deref(), puuid) {
            return a == b;
        }
        !game_name.is_empty()
            && self.game_name.eq_ignore_ascii_case(game_name)
            && self.tag_line.eq_ignore_ascii_case(tag_line)
    }
}

/// Secrets d'un compte, déchiffrés le temps d'une opération puis effacés.
/// (Les anciens champs, comme les copies de fichiers de session, sont ignorés à la lecture.)
#[derive(Debug, Clone, Default, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct AccountSecrets {
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
    /// Paire de jetons RSO propre à RiftSwitch pour ce compte (voir `riot::session`).
    #[serde(default)]
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub id_token: Option<String>,
}

#[cfg(test)]
impl AccountSecrets {
    pub fn with_password(p: &str) -> Self {
        Self {
            username: None,
            password: Some(p.to_string()),
            refresh_token: None,
            id_token: None,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RunePage {
    pub id: String,
    pub name: String,
    pub primary_style_id: i64,
    pub sub_style_id: i64,
    pub selected_perk_ids: Vec<i64>,
    pub champion: Option<String>,
    pub source_account: Option<String>,
    pub created_at: i64,
}
