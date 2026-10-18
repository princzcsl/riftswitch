# Modèle de menace

## Actifs

1. Sessions « Rester connecté » des comptes Riot (accès complet au compte tant qu'elles sont valides).
2. Identifiants Riot facultatifs (identifiant, mot de passe).
3. Métadonnées des comptes (emails, Riot ID, notes).

## Attaquants et protections

| Scénario | Protection | Limite |
|---|---|---|
| Vol du fichier `vault.rsv` (copie, sauvegarde cloud, disque perdu) | Chiffrement XChaCha20-Poly1305 ; avec mot de passe maître, Argon2id rend la force brute coûteuse | Mot de passe faible = force brute possible hors ligne |
| Autre utilisateur Windows sur le même PC | Fichiers dans le profil de l'utilisateur ; DPAPI liée à la session | Un administrateur peut toujours lire la session de l'utilisateur |
| Personne qui accède à la session Windows ouverte (mode sans mot de passe) | Aucune : la clé est déverrouillée par la session | Avertissement explicite au premier lancement ; mot de passe maître recommandé |
| Personne devant le PC, app ouverte | Verrouillage manuel, auto-verrouillage optionnel, verrouillage à la sortie de veille | Désactivé par défaut (choix produit) |
| Modification du coffre pour affaiblir le KDF ou altérer les données | En-tête dans l'AAD, bornes sur les paramètres | — |
| Fichier d'import / code de partage piégé | Codes et fichiers validés, décompression bornée à 4 Mio, chemins de profils filtrés | — |
| Logiciel malveillant déjà actif sur la session (keylogger, injection, lecture mémoire) | Fenêtre d'exposition réduite : secrets éphémères, effacement mémoire, verrouillage | **Aucune protection possible** : il peut capturer le mot de passe maître ou lire les fichiers de session du Riot Client, comme avec le Riot Client natif |
| Riot (anti-triche Vanguard) | Aucune interaction avec les processus de jeu : fichiers du Riot Client et API locales prévues pour les clients uniquement | Les API non officielles peuvent changer à chaque patch |

## Hors périmètre

- Compromission du compte Riot côté serveur.
- Sécurité du Riot Client lui-même.
