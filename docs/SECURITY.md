# Sécurité de RiftSwitch

## Coffre-fort (`src-tauri/src/vault`)

| Élément | Choix |
|---|---|
| Clé de coffre (DEK) | 256 bits aléatoires (`getrandom`) |
| Chiffrement | XChaCha20-Poly1305, nonce aléatoire de 192 bits par écriture |
| Mot de passe maître | Argon2id, 64 Mio, itérations calibrées pour ~500 ms (2 à 10), sel de 128 bits ; jamais stocké |
| Sans mot de passe | DEK confiée au Gestionnaire d'identifiants Windows (DPAPI) / Trousseau macOS via `keyring` |
| En-tête | mode + paramètres KDF inclus dans l'AAD : toute modification casse le déchiffrement |
| Bornes anti-DoS | paramètres Argon2 refusés hors bornes (64 Mio–1 Gio, t ≤ 32, p ≤ 4) |
| Secrets par compte | scellés une seconde fois avec AAD = id du compte : un secret copié sur un autre compte est refusé |
| Écriture | fichier temporaire + fsync + renommage, sauvegarde `.bak` ; copie avant migration de schéma |
| Mémoire | clés et secrets dans `Zeroizing` / `ZeroizeOnDrop`, effacés à la libération ; verrouiller = libérer la clé |
| Anti force brute | délai croissant après 3 échecs (jusqu'à 8 s) |

Tests : `cargo test vault` (aller-retour, mauvais mot de passe, falsification de l'en-tête et des données, aucun secret en clair sur disque, liaison secret ↔ compte, export / import, changement de mot de passe).

## Ce qui est stocké

- Sessions « Rester connecté » du Riot Client (fichiers `RiotGamesPrivateSettings.yaml`), chiffrées.
- Facultatif : identifiant et mot de passe Riot, chiffrés, utilisés uniquement pour se reconnecter si la session a expiré (expérimental).
- Email du compte, chiffré, affiché masqué (`pr***@gm***.com`).
- Les profils de config (touches, réglages) ne contiennent aucun secret : stockés en clair dans `profiles/`.

## Frontière front / cœur

Le front (webview) n'a jamais accès aux secrets. Il demande « jouer sur le compte X » ; le cœur Rust déchiffre la session, l'écrit dans le dossier du Riot Client, lance le jeu et efface le secret de la mémoire. Les commandes renvoient des vues sans secret (`account_view`).

## Réseau

- API locales prévues pour les clients Riot (Riot Client, LCU) en `https://127.0.0.1:<port>`, authentifiées par le mot de passe du lockfile. Leur certificat auto-signé est accepté uniquement pour ces appels locaux.
- Data Dragon (`ddragon.leagueoflegends.com`) : données publiques (versions, icônes, runes), sans compte.
- Mode hors ligne : relais vers `clientconfig.rpg.riotgames.com` et le serveur de chat Riot, uniquement quand l'utilisateur lance un jeu hors ligne.
- Aucune télémétrie n'est envoyée tant que l'utilisateur ne l'a pas activée (et l'envoi n'est pas encore implémenté).

## Exports

- Sauvegarde complète : fichier `.rsbackup` chiffré avec une phrase de passe dédiée (Argon2id + XChaCha20-Poly1305). Jamais en clair.
- Profils de config : code `RSC1.` (JSON compressé) ou fichier `.rsconfig`, sans secret.
- Runes : code `RSR1.` lisible, sans secret.

## Décisions

- Pas de partage de jeton de session à un ami : un jeton donne l'accès complet au compte, sans mot de passe ni A2F, et les CGU Riot interdisent le partage de compte. Non implémenté.
- Pas de page de connexion Riot embarquée qui verrait les identifiants : on utilise la fenêtre officielle du Riot Client.

## Signaler une faille

Ouvre une issue privée (Security advisory) sur le dépôt GitHub, sans détail exploitable public.
