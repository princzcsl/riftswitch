// Mises à jour signées publiées sur GitHub Releases. Le cœur télécharge le paquet, vérifie sa
// signature avec la clé publique intégrée, puis lance l'installateur silencieux.
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { check } from "@tauri-apps/plugin-updater";

const VERSION_KEY = "rs.lastVersion";

class UpdaterState {
  /** Version disponible, ou null si l'app est à jour / pas encore vérifiée. */
  available = $state<{ version: string; notes: string } | null>(null);
  /** Progression du téléchargement (0–1), null tant qu'on n'installe pas. */
  progress = $state<number | null>(null);
  /** Téléchargement fini : l'installateur tourne et va fermer puis relancer l'app. */
  installing = $state(false);
  checking = $state(false);
  dismissed = $state(false);
  /** Mise à jour automatique en cours : l'écran « Mise à jour en cours » couvre l'app. */
  updating = $state(false);
  /** Recherche de mise à jour au démarrage : l'écran de démarrage masque l'app. */
  startup = $state(!import.meta.env.DEV);

  /**
   * Version d'avant si l'app vient d'être mise à jour (une seule fois), sinon null.
   * La version courante est mémorisée pour le prochain démarrage.
   */
  justUpdated(current: string): string | null {
    try {
      const last = localStorage.getItem(VERSION_KEY);
      localStorage.setItem(VERSION_KEY, current);
      return last && last !== current ? last : null;
    } catch {
      return null;
    }
  }

  /**
   * Au démarrage : si une version est publiée, on l'installe sans rien demander.
   * `gate` : l'app n'est affichée qu'après la recherche (pas juste après une mise à jour).
   */
  async autoUpdate(gate = true) {
    if (import.meta.env.DEV) {
      this.startup = false;
      return;
    }
    if (!gate) this.startup = false;
    // Recherche limitée à 4 s pour ne jamais bloquer l'affichage.
    const search = this.check();
    const found = await Promise.race([search, new Promise<boolean>((r) => setTimeout(() => r(false), 4000))]);
    this.startup = false;
    if (!(found || (await search))) return;
    this.updating = true;
    try {
      await this.install();
    } catch {
      // Hors ligne, signature refusée : l'app reste utilisable normalement.
      this.updating = false;
    }
  }

  /** Mises à jour automatiques désactivées : pas d'écran de recherche au démarrage. */
  skipStartup() {
    this.startup = false;
  }

  /** Renvoie true si une mise à jour est disponible. Lève une erreur seulement si `manual`. */
  async check(manual = false): Promise<boolean> {
    if (this.checking) return !!this.available;
    this.checking = true;
    try {
      const u = await check({ timeout: 10_000 });
      this.available = u ? { version: u.version, notes: u.body ?? "" } : null;
      if (u) this.dismissed = false;
      return !!u;
    } catch (e) {
      if (manual) throw e;
      return false;
    } finally {
      this.checking = false;
    }
  }

  async install() {
    if (!this.available || this.progress !== null) return;
    this.progress = 0;
    const stop = await listen<[number, number | null]>("update-progress", (e) => {
      const [done, total] = e.payload;
      this.progress = total ? Math.min(done / total, 1) : 0;
    });
    try {
      await invoke("install_update");
      // L'app reste affichée jusqu'à ce que l'installateur la ferme pour copier ses fichiers.
      this.progress = 1;
      this.installing = true;
    } catch (e) {
      this.progress = null;
      throw e;
    } finally {
      stop();
    }
  }
}

export const updater = new UpdaterState();
