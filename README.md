# RiftSwitch

Launcher multi-comptes Riot Games (League of Legends, TFT, VALORANT) pour Windows (et macOS à venir).
Choisis un compte, clique sur **Jouer**, choisis le jeu : RiftSwitch gère la bascule de session, la langue du jeu, ta config de touches et le mode hors ligne.

- **Coffre chiffré** : XChaCha20-Poly1305, mot de passe maître dérivé par Argon2id (~500 ms, 64 Mio) ou clé confiée à Windows / macOS. Rien ne quitte la machine.
- **Bascule de compte en un clic** : identifiant et mot de passe stockés chiffrés, connexion à Riot sans saisie (tu résous seulement le captcha ou le code A2F quand Riot les demande), puis lancement direct du jeu choisi, avec les étapes affichées.
- **Gestion des comptes** : surnom, icône (champions, agents VALORANT ou icône d'invocateur automatique), couleur, dossiers, favoris, tags, rôles principaux, langue de jeu par compte, notes. Rangs, niveau et région récupérés automatiquement.
- **Configs de jeu** : profils LoL (touches, interface, son, graphismes) et VALORANT, éditeur de touches avec détection des conflits, partage par code ou fichier, application automatique par compte.
- **Mode hors ligne** : proxy XMPP local inspiré de [Deceive](https://github.com/molenzwiebel/deceive), sans message automatique.
- **Interface** Liquid Glass multi-thème (JSON), mode performance, zoom 80–150 %, 5 langues complètes (en, fr, es, it, de) + structure prête pour 9 autres.

La conception complète est dans [docs/CONCEPTION.md](docs/CONCEPTION.md), la feuille de route dans [docs/ROADMAP.md](docs/ROADMAP.md).

## Installation

Télécharge `RiftSwitch_x.y.z_x64-setup.exe` depuis la page [Releases](../../releases) et lance-le (Windows 10/11, WebView2 installé automatiquement si absent).
L'installateur n'est pas encore signé : Windows SmartScreen peut afficher « Windows a protégé votre ordinateur » → **Informations complémentaires** → **Exécuter quand même**.

## Stack

| Couche | Choix |
|---|---|
| Cœur | Rust, Tauri 2 |
| Interface | Svelte 5, TypeScript, Vite, CSS natif (jetons de thème) |
| Crypto | `argon2`, `chacha20poly1305`, `zeroize`, `keyring` (DPAPI / Trousseau) |
| Réseau | `reqwest` + `rustls` (API locales Riot en 127.0.0.1, Data Dragon) |
| Hors ligne | `tokio-rustls`, `rcgen`, `tiny_http` |

## Développement

Prérequis : Node 20+, Rust stable, et sous Windows les outils C++ de Visual Studio.

```bash
npm install
```

Interface seule, avec des comptes fictifs (aucun accès au Riot Client ni au coffre) :

```bash
npm run dev
```

Puis ouvre http://localhost:1420 (paramètres `?theme=hextech`, `?lang=en`, `?view=setup`, `?view=unlock`).

Application complète, données isolées dans `.devdata` (PowerShell) :

```powershell
$env:RIFTSWITCH_DATA_DIR = "$PWD\.devdata"; npm run app:dev
```

Tests et vérifications :

```bash
cd src-tauri && cargo test
```

```bash
npm run check
```

Build release (exe + installateur NSIS dans `src-tauri/target/release/bundle/nsis`) :

```bash
npm run app:build
```

Version portable : place un fichier vide `portable.flag` à côté de `riftswitch.exe`. Les données vont alors dans `./data` et le mot de passe maître devient obligatoire.

## Structure

```
src/                 Interface Svelte
  pages/             Comptes, Ce soir, Configs, Runes, Boutiques, Patchs, Esport, Succès, Paramètres, Setup, Unlock
  components/        Cartes, bouton Jouer, éditeurs de touches, modales…
  lib/               État, API, i18n, thèmes, touches, recommandation
  locales/           Un JSON par langue
  themes/            Un JSON par thème
src-tauri/src/
  vault/             Coffre chiffré (crypto, modèle)
  riot/              Riot Client : chemins, processus, API locale, session, lancement, LCU
  configs/           Profils de config (INI, LoL, Valorant, codes)
  offline.rs         Proxy hors ligne
  runes.rs           Pages de runes
  monitor.rs         Surveillance de fond (compte connecté, rangs, temps de jeu, verrouillage auto)
  tray.rs            Zone de notification
  commands/          Commandes exposées au front
docs/                CONCEPTION, SECURITY, THREAT_MODEL, PRIVACY
```

## Licence

RiftSwitch est un logiciel libre sous licence [GPL-3.0-or-later](LICENSE) : tu peux l'utiliser, l'étudier, le modifier et le redistribuer, à condition que toute version redistribuée reste sous la même licence, avec son code source.
Le nom « RiftSwitch » et son logo ne sont pas couverts par la licence : une version modifiée doit porter un autre nom.
Travaux tiers intégrés ou adaptés : voir [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

## Avertissement

RiftSwitch n'est pas approuvé par Riot Games et ne reflète pas les opinions de Riot Games ni de quiconque participant officiellement à la production ou à la gestion des propriétés de Riot Games. Riot Games et toutes les propriétés associées sont des marques commerciales ou déposées de Riot Games, Inc.
Certaines fonctions reposent sur des interfaces non officielles : elles affichent leur niveau de risque et sont désactivées par défaut.
