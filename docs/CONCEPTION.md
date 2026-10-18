# RiftSwitch — Fiche projet

Oct 17, 2026 · @Prince

## Vision et piliers

RiftSwitch est un launcher desktop (Windows x64/arm64, macOS) qui remplace le passage par le Riot Client : on choisit un compte, on clique « Jouer », on choisit LoL, Valorant ou TFT, et l'app gère connexion, config, langue du jeu et mode hors ligne. Ce n'est pas un gestionnaire de mots de passe : l'interface doit ressembler à un launcher de jeu, pas à un coffre-fort.

| Pilier | Ce que ça veut dire concrètement |
| --- | --- |
| Sécurité | Identifiants chiffrés au repos (jamais en clair, jamais envoyés hors de la machine), secrets éphémères en mémoire, partage sans divulguer email/mot de passe |
| Ergonomie | Lancer une partie sur le bon compte en 2 clics maximum depuis l'écran d'accueil, la zone de notification ou la Jump List |
| Optimisation | App fluide sur une petite config ; quasi aucune ressource consommée quand elle tourne en fond pendant une partie |

Nom retenu : **RiftSwitch**. Public : joueurs multi-comptes LoL / Valorant / TFT, streamers, coachs.

Cibles chiffrées à valider : RAM < 120 Mo fenêtre ouverte, < 40 Mo en fond ; CPU \~0 % en fond ; démarrage à froid < 1,5 s ; binaire < 20 Mo.

## Points de vigilance avant de coder

Plusieurs fonctionnalités reposent sur des API non officielles de Riot : elles peuvent casser à chaque patch et certaines exposent l'utilisateur à une sanction. Chaque fonctionnalité porte donc un niveau de risque affiché dans l'app.

| Fonctionnalité | Source technique | Risque | Décision |
| --- | --- | --- | --- |
| Switch de compte, lancement des jeux | Riot Client local (fichiers de session, API locale) | Faible, toléré de fait | Cœur du produit |
| Mode hors ligne | Proxy XMPP local façon Deceive | Faible (Deceive existe depuis des années) | À faire, sans les messages automatiques |
| Rangs, decay, pénalités | API locale LCU / Riot Client quand le compte est connecté, cache local ensuite ; API tierce en option | Faible à moyen | Cache local + API tierce opt-in |
| Aperçu boutique Valorant, Marché nocturne | Endpoints non publics du store avec le jeton du compte | Moyen (CGU) | Opt-in avec avertissement |
| Boutique fantastique LoL | Identique pour tous : une seule lecture suffit | Faible | Sans compte, données publiques ou client |
| Partage de compte à un ami (jeton) | Transfert de session | **Élevé** : les CGU Riot interdisent le partage de compte | Voir ci-dessous |

**Partage à un ami.** Tu as choisi « les deux » (mes appareils + amis). Je recommande de le garder, mais : désactivé par défaut, avertissement explicite rappelant que le partage de compte enfreint les CGU Riot et peut mener à une sanction, durée d'expiration courte, révocable. Le transfert vers ses propres appareils reste la voie mise en avant.

**Clé API de production Riot.** La politique développeur Riot exige que [le produit soit enregistré et approuvé sur le Developer Portal](https://developer.riotgames.com/docs/lol), et l'accès aux données d'un joueur passe par [RSO, le système de connexion officiel](https://developer.riotgames.com/docs/lol). Pour Valorant, [toute app doit faire consentir l'utilisateur au partage de ses données](https://developer.riotgames.com/docs/Valorant). Un outil dont le cœur est de stocker des identifiants et de basculer entre comptes a peu de chances d'être approuvé tel quel. Plan : demander une clé pour un usage « statistiques via RSO » uniquement, et ne jamais en dépendre ; l'app doit fonctionner sans.

**Vanguard et anti-triche.** L'app ne doit jamais lire ni écrire dans la mémoire d'un processus de jeu, injecter du code ou automatiser des entrées en jeu. Elle agit uniquement avant le lancement (fichiers, Riot Client) et via les API locales prévues pour les clients.

## Stack technique et architecture

Recommandation : **Tauri 2 (cœur en Rust) + interface web légère (Svelte 5 + TypeScript + Vite)**. Tauri utilise le moteur web du système (WebView2 sous Windows, WKWebView sous macOS) au lieu d'embarquer Chromium : binaire et RAM bien plus faibles qu'Electron, et Rust pour tout ce qui touche aux secrets. Le dossier racine sera donc `Rust/RiftSwitch/` (le « dossier du langage » demandé).

| Couche | Choix | Pourquoi |
| --- | --- | --- |
| Cœur | Rust, Tauri 2 | Mémoire sûre, binaire \~10 Mo, x64 + arm64 + macOS universel |
| Interface | Svelte 5, TypeScript, CSS natif (variables de thème) | Pas de virtual DOM, bundle minimal, fluide sur petite config |
| Crypto | `argon2` (Argon2id), `chacha20poly1305` (XChaCha20-Poly1305), `zeroize`, `secrecy` | Standards actuels, effacement mémoire des secrets |
| Clé machine | Windows DPAPI / Credential Manager, macOS Keychain via `keyring` | Coffre lié à l'utilisateur de la session si pas de mot de passe maître |
| Stockage | SQLite (`rusqlite` + SQLCipher) ou fichier unique chiffré | Données de comptes, dossiers, tags, cache |
| Réseau | `reqwest` + `rustls` | TLS sans OpenSSL |
| Mode hors ligne | Proxy XMPP local en Rust inspiré de Deceive | Pas de dépendance .NET |
| i18n | Fichiers JSON par langue, `svelte-i18n` ou équivalent léger | Ajout d'une langue sans recompiler la logique |
| Mises à jour | `tauri-plugin-updater`, paquets signés, GitHub Releases | Silencieuses, vérifiées par signature |
| Discord | Rich Presence via IPC local (`discord-rich-presence`) |  |
| Télémétrie | Aptabase (plugin Tauri, hébergé en UE) ou endpoint maison | Anonyme, opt-in, compatible RGPD |
| CI | GitHub Actions : build Windows x64/arm64, macOS universel, portable | Signature de code Windows + notarisation Apple |

Organisation du dépôt (GitHub privé `RiftSwitch`) : `src-tauri/` (Rust : `vault`, `riot_client`, `lcu`, `xmpp_proxy`, `config_sync`, `share`, `telemetry`), `src/` (Svelte : `routes`, `components`, `themes`, `i18n`), `docs/` (`CONCEPTION.md`, `SECURITY.md`, `THREAT_MODEL.md`, `PRIVACY.md`).

Principe clé : le front n'a jamais accès aux mots de passe. Il demande « connecter le compte X » au cœur Rust, qui déchiffre, utilise et efface le secret.

## Sécurité

Correction importante : les mots de passe Riot ne peuvent pas être **hashés**, car l'app doit pouvoir les réutiliser pour se connecter. Ils sont **chiffrés**. Seul le mot de passe maître est dérivé (Argon2id) et n'est jamais stocké.

### Coffre-fort

- Chiffrement : XChaCha20-Poly1305, une clé de coffre aléatoire de 256 bits.
- Avec mot de passe maître (recommandé) : clé dérivée par Argon2id (paramètres calibrés à \~500 ms sur la machine, au moins 64 Mo de mémoire), qui protège la clé de coffre.
- Sans mot de passe maître : clé de coffre protégée par DPAPI (Windows) ou le Trousseau (macOS). Avertissement clair au premier lancement : quiconque accède à la session Windows/macOS accède aux comptes.
- Auto-verrouillage : désactivé par défaut, activable (délai au choix, verrouillage à la mise en veille).
- Secrets en mémoire : types `secrecy`/`zeroize`, durée de vie minimale, jamais journalisés, jamais transmis au front.
- Sessions Riot : les jetons et cookies de session capturés sont chiffrés comme les mots de passe.
- Fichiers : écriture atomique, sauvegarde chiffrée automatique avant migration de schéma.

### Ajout de compte

- Par défaut : connexion Riot dans l'app (fenêtre de connexion officielle Riot dans une webview isolée), A2F gérée si demandée.
- Option séparée : « Ajouter le compte actuellement connecté au Riot Client » (capture de session).

### Exports et partage

| Export | Contenu | Protection |
| --- | --- | --- |
| Config de l'app avec comptes | Coffre complet | Fichier chiffré avec une phrase de passe dédiée (Argon2id), jamais en clair |
| Config de jeu, runes | Pas de secret | Fichier ou code court copiable |
| Transfert vers un autre appareil | Session ou identifiants | Code + QR code, chiffrement de bout en bout (échange de clés X25519, code de vérification à 6 chiffres), usage unique, expiration choisie |
| Partage à un ami | Session uniquement, jamais email ni mot de passe | Même canal, expiration courte, révocable, avertissement CGU |

Le transfert direct entre appareils se fait de préférence en réseau local ; sinon via un relais qui ne voit que des données chiffrées.

### Audit des comptes

- Rappel si l'A2F n'est pas activée sur un compte.
- Vérification Have I Been Pwned : **opt-in**, en k-anonymat (seuls les 5 premiers caractères du hash SHA-1 du mot de passe sortent, jamais l'email ni le mot de passe). Texte explicatif simple, car l'utilisateur peut croire que ses identifiants quittent la machine.

### Modèle de menace (à afficher dans l'app et dans `SECURITY.md`)

Un logiciel malveillant déjà actif sur la session (keylogger, injection, lecture mémoire) peut capturer le mot de passe maître ou les secrets pendant l'utilisation. Aucun gestionnaire local ne protège contre ça, et c'est aussi vrai avec le Riot Client natif. RiftSwitch réduit la fenêtre d'exposition (auto-verrouillage, secrets éphémères, effacement mémoire), mais ne la supprime pas.

### Réinitialisation

Paramètre « Réinitialiser l'app » : double confirmation, effacement de toutes les données (coffre, cache, préférences, entrées Trousseau/DPAPI), retour au premier lancement.

## Fonctionnalités par module

### 1. Comptes

- Liste des comptes : email partiellement masqué (`pr***@gm***.com`), Riot ID, rang LoL / Valorant / TFT, date d'ajout, région, icône, couleur, tags, dossier.
- Dossiers pour classer ; dans la vue « Tous les dossiers », chaque compte affiche son dossier.
- Icône par compte : set inspiré LoL (champions, icônes chibi) ou import d'image avec recadrage circulaire intégré.
- Tags personnalisés, pool de champions / agents par compte.
- Historique des anciens pseudos (Riot ID) connus, alimenté à chaque connexion.
- Santé du compte : pénalités, restrictions de file, si lisibles via le client.
- Compte à rebours de decay (Diamant+ LoL ; Valorant si le système existe) et de fin de saison.
- Détection du compte actuellement connecté au Riot Client.
- Bouton **Jouer** sur chaque compte → menu avec icônes (et texte en option) : League of Legends, Valorant, TFT. Option « en ligne / hors ligne » dans ce même menu.
- Langue de jeu par compte et par jeu (ex. Valorant en japonais, LoL en anglais), choisie à l'ajout, modifiable ensuite.

### 2. Configs de jeu

- Profils de config LoL et Valorant : touches, HUD, son, graphismes.
- Bascule rapide entre profils, duplication des touches d'un compte vers un autre.
- Import / export de config (fichier ou code).
- Runes LoL : import / export entre comptes et code court à partager.
- Valorant : les paramètres sont stockés côté serveur ; passer par l'API locale du client ou les fichiers de cache selon ce qui est faisable.

### 3. Mode hors ligne

- Proxy XMPP local inspiré de [Deceive](https://github.com/molenzwiebel/deceive) : apparaître hors ligne pour les amis tout en jouant.
- Pas de message automatique envoyé dans le chat du client.
- Seule notification système de l'app : confirmation du mode hors ligne au lancement d'un jeu.
- Option (activée par défaut) pour couper Discord Rich Presence en mode hors ligne.

### 4. Infos de jeu

- Aperçu boutique Valorant + Marché nocturne (opt-in, avertissement).
- Boutique fantastique LoL : rotation commune à tous les joueurs toutes les 14 jours (skins prestige et hextech, icônes et chromas rares), affichée avec le compte à rebours de rotation.
- Alertes de patch avec mini-récap buffs / nerfs (LoL, Valorant, TFT), dans l'app uniquement.
- Hub esport synthétique : matchs du jour, résultats, compétitions suivies (LEC, LCK, VCT…).

### 5. « Sur quel compte jouer ce soir ? »

L'utilisateur choisit un objectif : **Grind** (libellé français), s'entraîner sur un nouveau rôle, jouer avec des amis d'un rang donné. L'app propose le compte le plus adapté selon rang, decay imminent, pool de champions, pénalités et temps de jeu récent.

### 6. Temps de jeu

Compteur d'heures jouées par compte et par jeu quand l'app est active, y compris en mode hors ligne.

### 7. Succès et cosmétiques

Succès inter-comptes et défis propres à l'app, qui rapportent une monnaie interne. Cette monnaie sert uniquement à débloquer des thèmes et cosmétiques de l'app. Aucun achat avec de l'argent réel.

### 8. Intégrations système

- L'app tourne en fond, icône dans la zone de notification Windows / barre de menus macOS.
- Jump List Windows : comptes favoris, lancement direct.
- Discord Rich Presence avec options (afficher le Riot ID, le type de partie, le jeu…).
- Mode streamer : masque emails, Riot ID, rangs détaillés et boutiques dans l'interface.
- Mises à jour automatiques silencieuses, appliquées au prochain démarrage.
- Version portable (dossier unique, coffre à côté de l'exécutable, sans DPAPI par défaut → mot de passe maître obligatoire).
- Mise à l'échelle : respect du réglage DPI Windows / macOS + curseur de zoom dans les paramètres (80 % à 150 %).
- Support TFT sur l'ensemble des modules (rang, lancement, patchs).

## UI/UX, thèmes et langues

Objectif : l'app doit ressembler à un launcher de jeu (type Porofessor, Blitz), pas à un gestionnaire de mots de passe.

### Ergonomie

- Écran d'accueil = grille de cartes de comptes (icône, Riot ID, rangs, couleur), bouton **Jouer** bien visible, pas de champs ni de listes d'identifiants.
- Les détails sensibles (email, sécurité) sont dans une fiche secondaire, pas en façade.
- Barre latérale : Comptes, Configs, Boutiques, Patchs, Esport, Succès, Paramètres.
- Fenêtre redimensionnable, mode plein écran « vrai » supprimé (maximisation classique uniquement).
- Premier lancement guidé : langue, thème, mot de passe maître (recommandé) ou non (avertissement), import éventuel, ajout du premier compte, consentement télémétrie.

### Thèmes

- Système multi-thème par variables CSS (couleurs, flou, transparence, rayons) ; un thème = un fichier JSON, ajout sans toucher au code.
- Thème par défaut Liquid Glass : verre dépoli (`backdrop-filter`), reflets, bordures lumineuses ; sous Windows 11, effet Mica/Acrylic natif via `window-vibrancy`, vibrancy native sous macOS.
- Thème « full Liquid Glass » inspiré de l'extension [ig-utils-vibe](https://github.com/princzcsl/ig-utils-vibe) à prototyper.
- Avant de figer : produire 3 à 4 maquettes de thème (ex. Liquid Glass clair, Liquid Glass sombre, Hextech or/bleu, Néon Valorant) pour que tu choisisses.
- Mode performance : désactive flou et animations automatiquement sur petite config ou batterie, et manuellement dans les paramètres.

### Icône de l'app

Icône originale dans l'esprit LoL (personnage chibi). À créer comme personnage original : utiliser un champion Riot exact pose un problème de droits pour une app distribuée. Riot autorise certains usages via sa politique « Legal Jibber Jabber », à vérifier avant d'utiliser des assets officiels.

### Langues

Anglais, français, espagnol, italien, allemand, plus portugais (Brésil), polonais, turc, russe, coréen, japonais, chinois traditionnel, vietnamien, arabe (interface RTL à prévoir). Ces langues correspondent aux régions serveurs majeures de LoL/Valorant. Langue détectée depuis le système au premier lancement.

## Exclusions et notifications

Ce que RiftSwitch ne fait **pas** :

- Pas de raccourcis macro, pas de touche ou bouton panique.
- Pas d'historique local des LP / RR.
- Pas de lecture ou écriture de mémoire des jeux, pas d'injection, pas d'automatisation en jeu.
- Pas d'achat avec de l'argent réel.
- Pas de vrai plein écran.

Notifications système : **une seule**, la confirmation du mode hors ligne au lancement d'un jeu en hors ligne. Tout le reste (patchs, decay, boutiques, succès) reste visible dans l'app uniquement, sous forme de badges.

## Télémétrie et RGPD

La télémétrie est anonyme, minimale et **opt-in** (case non cochée par défaut au premier lancement), car la base légale la plus sûre sous RGPD pour un usage non indispensable est le consentement.

| Statistique | Forme envoyée |
| --- | --- |
| Installations, utilisateurs uniques | Identifiant d'installation aléatoire, régénérable, sans lien avec un compte Riot |
| Nombre de comptes enregistrés | Tranche (1, 2–5, 6–10, 10+), pas le chiffre exact |
| Lancements LoL / Valorant / TFT via l'app | Compteur agrégé |
| Heures jouées avec l'app lancée | Total arrondi à l'heure |
| Version, OS, architecture, langue, thème | Valeurs brutes |

Règles :

- Jamais d'email, de Riot ID, de PUUID, d'IP conservée ni de données de partie.
- Envoi groupé une fois par jour maximum.
- Écran « Voir ce qui est envoyé » dans les paramètres, retrait du consentement en un clic.
- Hébergement en UE (Aptabase EU ou serveur maison), durée de conservation définie (ex. 13 mois).
- Politique de confidentialité `PRIVACY.md` en plusieurs langues ; mention des autres régimes (CCPA Californie, LGPD Brésil, PIPA Corée).
- Les stats publiques (site, GitHub, Discord) sont produites à partir des agrégats uniquement.

## Feuille de route et critères d'acceptation

Le travail avance en 10 phases regroupées en 3 jalons ; aucune phase ne démarre sans validation de la précédente. Pas de dates : le rythme dépend de ton temps disponible.

```
Fondations (→ Alpha privée)      Cœur produit (→ Bêta publique)    Contenu (→ v1.0)
0 · Mise en place                4 · Mode hors ligne               8 · Patchs, esport
1 · Coffre-fort                  5 · Configs de jeu                9 · Audit, télémétrie
2 · Comptes et switch            6 · Données de jeu
3 · Shell et thèmes              7 · Partage, exports
```

Le coffre-fort (phase 1) est construit et testé avant tout écran de compte, pour que rien ne soit jamais stocké en clair, même temporairement.

| Jalon | Critères pour passer |
| --- | --- |
| Alpha privée | Ajout de compte par connexion Riot et par capture ; switch + lancement LoL/Valorant/TFT en 2 clics ; coffre chiffré testé (aucun secret en clair sur disque ni dans les logs) ; budgets RAM/CPU tenus ; builds Windows x64/arm64 et macOS verts |
| Bêta publique | Hors ligne fiable sur LoL et Valorant ; configs et runes importables ; rangs et decay affichés avec cache ; partage par code/QR chiffré de bout en bout ; mises à jour silencieuses signées ; 5 langues complètes |
| v1.0 | Patchs, esport, succès ; audit sécurité ; télémétrie opt-in documentée dans `PRIVACY.md` ; version portable ; revue de sécurité externe ou relecture indépendante du coffre |

## Idées de fonctionnalités supplémentaires

Propositions à valider, aucune n'est dans le périmètre tant que tu ne l'as pas choisie.

- [ ] **Palette de commandes** (Ctrl+K / Cmd+K) : taper « smurf euw » puis Entrée pour lancer.
- [ ] **Lancement planifié** : « lance LoL hors ligne sur le compte X à 21 h ».
- [ ] **Fermeture propre** : fermer le Riot Client et les jeux avant un switch, avec détection d'une partie en cours pour bloquer le switch.
- [ ] **Notes par compte** (chiffrées) : objectifs, coach, rappels.
- [ ] **Rappel d'expiration de récompenses** : missions, pass de combat, coffres Hextech à ouvrir.
- [ ] **Résumé de session** à la fermeture du jeu : temps joué, nombre de parties.
- [ ] **Profil « nouvelle saison »** : appliquer la même config de touches à tous les comptes en un clic après un reset.
- [ ] **Sauvegarde chiffrée automatique** du coffre vers un dossier choisi (OneDrive, iCloud), restauration guidée.
- [ ] **Vue « amis multi-comptes »** : quels amis sont en ligne, sur quel compte (lecture seule, via XMPP local).
- [ ] **Indicateur d'état des serveurs Riot** par région, avant de lancer.
- [ ] **Mode coach / LAN** : prêter un poste pour quelques heures avec verrouillage auto à la fin.
