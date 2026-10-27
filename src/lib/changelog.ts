// Nouveautés de chaque version, affichées une fois après une mise à jour (et depuis « À propos »).
// La version anglaise sert aussi de notes de release sur GitHub.

type Notes = { added?: string[]; changed?: string[]; fixed?: string[] };
export type Release = { version: string; notes: Record<string, Notes> };

export const CHANGELOG: Release[] = [
  {
    version: "1.3.0",
    notes: {
      fr: {
        added: [
          "Palette de commandes Ctrl+K : lance un compte sur un jeu, ouvre une page, active le mode fantôme ou verrouille l'app au clavier.",
          "Lancements planifiés : un compte et un jeu lancés automatiquement à l'heure et aux jours choisis.",
          "Réutilisation de la session Riot enregistrée : la plupart des bascules se font sans captcha.",
          "Configs : réglages rangés comme dans le jeu et traduits, curseurs pour le son, profils de viseur VALORANT.",
          "Configs : application à plusieurs comptes d'un coup, sauvegarde d'origine automatique et bouton Restaurer.",
          "Fenêtre des nouveautés après chaque mise à jour.",
        ],
        changed: [
          "Le mode fantôme peut encore être changé pendant un lancement, jusqu'au démarrage du Riot Client.",
          "Le profil de config indique son compte d'origine (photo et Riot ID), même partagé à un ami.",
        ],
        fixed: ["Noms de comptes vides dans les écrans de config.", "Noms de certaines touches VALORANT."],
      },
      en: {
        added: [
          "Ctrl+K command palette: launch an account on a game, open a page, toggle ghost mode or lock the app from the keyboard.",
          "Scheduled launches: an account and a game started automatically at the chosen time and days.",
          "Reuse of the saved Riot session: most switches no longer need a captcha.",
          "Configs: settings laid out like in game and translated, sliders for sound, VALORANT crosshair profiles.",
          "Configs: apply to several accounts at once, automatic original backup and a Restore button.",
          "What's new window after every update.",
        ],
        changed: [
          "Ghost mode can still be changed during a launch, until the Riot Client starts.",
          "Config profiles show the account they come from (picture and Riot ID), even when shared with a friend.",
        ],
        fixed: ["Empty account names in the config screens.", "Names of some VALORANT keybinds."],
      },
      de: {
        added: [
          "Befehlspalette Strg+K: Konto in einem Spiel starten, Seite öffnen, Geistermodus umschalten oder die App sperren.",
          "Geplante Starts: Konto und Spiel werden zur gewählten Zeit an den gewählten Tagen gestartet.",
          "Wiederverwendung der gespeicherten Riot-Sitzung: die meisten Wechsel brauchen kein Captcha mehr.",
          "Konfigurationen: Einstellungen wie im Spiel geordnet und übersetzt, Regler für Ton, VALORANT-Fadenkreuzprofile.",
          "Konfigurationen: auf mehrere Konten gleichzeitig anwenden, automatische ursprüngliche Sicherung und Wiederherstellen.",
          "Fenster mit Neuigkeiten nach jedem Update.",
        ],
        changed: [
          "Der Geistermodus kann während eines Starts noch geändert werden, bis der Riot Client startet.",
          "Konfigurationsprofile zeigen ihr Ursprungskonto (Bild und Riot ID), auch wenn sie geteilt werden.",
        ],
        fixed: ["Leere Kontonamen in den Konfigurationsansichten.", "Namen einiger VALORANT-Tasten."],
      },
      es: {
        added: [
          "Paleta de comandos Ctrl+K: inicia una cuenta en un juego, abre una página, activa el modo fantasma o bloquea la app.",
          "Inicios programados: una cuenta y un juego iniciados automáticamente a la hora y los días elegidos.",
          "Reutilización de la sesión de Riot guardada: la mayoría de los cambios ya no piden captcha.",
          "Configuraciones: ajustes ordenados como en el juego y traducidos, deslizadores de sonido, perfiles de mira de VALORANT.",
          "Configuraciones: aplicar a varias cuentas a la vez, copia original automática y botón Restaurar.",
          "Ventana de novedades tras cada actualización.",
        ],
        changed: [
          "El modo fantasma aún se puede cambiar durante un inicio, hasta que arranca el Riot Client.",
          "Los perfiles de configuración muestran su cuenta de origen (imagen y Riot ID), incluso compartidos.",
        ],
        fixed: ["Nombres de cuenta vacíos en las pantallas de configuración.", "Nombres de algunas teclas de VALORANT."],
      },
      it: {
        added: [
          "Tavolozza comandi Ctrl+K: avvia un account su un gioco, apri una pagina, attiva la modalità fantasma o blocca l'app.",
          "Avvii pianificati: un account e un gioco avviati automaticamente all'ora e nei giorni scelti.",
          "Riutilizzo della sessione Riot salvata: la maggior parte dei cambi non chiede più il captcha.",
          "Configurazioni: impostazioni ordinate come nel gioco e tradotte, cursori per l'audio, profili mirino di VALORANT.",
          "Configurazioni: applica a più account insieme, backup originale automatico e pulsante Ripristina.",
          "Finestra delle novità dopo ogni aggiornamento.",
        ],
        changed: [
          "La modalità fantasma si può ancora cambiare durante un avvio, finché il Riot Client non parte.",
          "I profili di configurazione mostrano l'account d'origine (immagine e Riot ID), anche se condivisi.",
        ],
        fixed: ["Nomi degli account vuoti nelle schermate di configurazione.", "Nomi di alcuni tasti di VALORANT."],
      },
    },
  },
];

const parts = (v: string) => v.split(".").map((n) => Number(n) || 0);
function newer(a: string, b: string): boolean {
  const [x, y] = [parts(a), parts(b)];
  for (let i = 0; i < 3; i++) if ((x[i] ?? 0) !== (y[i] ?? 0)) return (x[i] ?? 0) > (y[i] ?? 0);
  return false;
}

/** Versions publiées après `from` et jusqu'à `current` (toutes si `from` est vide), la plus récente en premier. */
export function releasesSince(from: string, current: string): Release[] {
  return CHANGELOG.filter((r) => !newer(r.version, current) && (!from || newer(r.version, from)));
}
