// Mises à jour signées publiées sur GitHub Releases. Le paquet est vérifié avec la clé publique
// intégrée à l'app avant installation ; seule la requête de vérification sort vers github.com.
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

class UpdaterState {
  /** Version disponible, ou null si l'app est à jour / pas encore vérifiée. */
  available = $state<{ version: string; notes: string } | null>(null);
  /** Progression du téléchargement (0–1), null tant qu'on n'installe pas. */
  progress = $state<number | null>(null);
  checking = $state(false);
  dismissed = $state(false);
  #update: Update | null = null;

  /** Renvoie true si une mise à jour est disponible. Lève une erreur seulement si `manual`. */
  async check(manual = false): Promise<boolean> {
    if (this.checking) return !!this.available;
    this.checking = true;
    try {
      const u = await check();
      this.#update = u ?? null;
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
    const u = this.#update;
    if (!u || this.progress !== null) return;
    let total = 0;
    let done = 0;
    this.progress = 0;
    try {
      await u.downloadAndInstall((ev) => {
        if (ev.event === "Started") total = ev.data.contentLength ?? 0;
        else if (ev.event === "Progress") {
          done += ev.data.chunkLength;
          this.progress = total ? Math.min(done / total, 1) : 0;
        } else this.progress = 1;
      });
      // Sous Windows l'installateur ferme l'app lui-même ; ailleurs on relance.
      await relaunch();
    } catch (e) {
      this.progress = null;
      throw e;
    }
  }
}

export const updater = new UpdaterState();
