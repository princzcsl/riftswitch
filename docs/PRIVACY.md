# Politique de confidentialité

*Version française. Les autres langues suivront.*

## En bref

RiftSwitch fonctionne entièrement sur ton ordinateur. Tes identifiants, sessions et données de compte ne sont jamais envoyés à nos serveurs (nous n'en avons pas).

## Données stockées localement

- Coffre chiffré : comptes, sessions Riot, identifiants facultatifs, notes, rangs mis en cache, temps de jeu.
- Préférences (thème, langue, options) en clair dans `settings.json`.
- Profils de config de jeu (touches, réglages) en clair dans `profiles/`.

## Connexions réseau

| Destination | Quand | Ce qui est envoyé |
|---|---|---|
| `127.0.0.1` (Riot Client, client LoL) | En permanence quand le client tourne | Requêtes locales, rien ne sort de la machine |
| `authenticate.riotgames.com` | À la bascule vers un compte | Identifiant et mot de passe du compte (TLS), envoyés uniquement à Riot, comme le ferait le Riot Client |
| `ddragon.leagueoflegends.com`, `raw.communitydragon.org`, `valorant-api.com` | Affichage d'icônes (champions, agents), runes, patchs | Requêtes publiques, sans compte |
| `github.com` | Au démarrage si « Rechercher les mises à jour » est activé, ou sur demande | Requête publique du manifeste de version ; le paquet téléchargé est vérifié par signature avant installation |
| Serveurs Riot de configuration et de chat | Uniquement en mode hors ligne | Le trafic que le Riot Client enverrait de toute façon, relayé localement |

## Statistiques anonymes (opt-in)

Désactivées par défaut. Si tu les actives, au plus une fois par jour :

| Statistique | Forme |
|---|---|
| Installation | identifiant aléatoire régénérable, sans lien avec un compte Riot |
| Nombre de comptes | tranche (1, 2–5, 6–10, 10+) |
| Lancements LoL / TFT / VALORANT | compteurs agrégés |
| Heures jouées | total arrondi à l'heure |
| Version, système, architecture, langue, thème | valeurs brutes |

Jamais d'email, de Riot ID, de PUUID, d'adresse IP conservée ni de données de partie. L'écran « Voir ce qui est envoyé » des paramètres montre exactement ces données. Retrait du consentement en un clic.

Base légale : consentement (RGPD art. 6.1.a). Hébergement prévu dans l'UE, conservation 13 mois. Droits d'accès et d'effacement : l'identifiant d'installation se régénère depuis les paramètres.

Autres régimes : CCPA (Californie), LGPD (Brésil), PIPA (Corée) : aucune vente ni partage de données personnelles.

**État actuel** : le consentement est enregistré, mais aucun envoi n'est encore implémenté.

## Effacement

Paramètres › Avancé › Réinitialiser l'app : efface coffre, sessions, profils, cache, préférences et la clé conservée par le système.
