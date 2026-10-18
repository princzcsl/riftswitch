// État global réactif de l'interface (Svelte 5 runes).
import { listen } from "@tauri-apps/api/event";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { api } from "./api";
import { latestVersion } from "./ddragon";
import { detectLanguage, errorText, setLanguage, t } from "./i18n.svelte";
import { applyTheme, autoPerformance } from "./themes";
import type { Account, Bootstrap, ConfigProfile, Folder, Game, RiotStatus, Settings } from "./types";

export type Page = "accounts" | "tonight" | "configs" | "runes" | "shops" | "patches" | "esport" | "achievements" | "settings";

export interface Toast {
  id: number;
  kind: "info" | "success" | "error" | "warning";
  text: string;
}

const emptyStatus: RiotStatus = {
  clientInstalled: false,
  procs: { riotClient: false, riotClientPid: null, lolClient: false, lolGame: false, valGame: false },
  identity: null,
  activeAccountId: null,
  proxyRunning: false,
  offline: false,
  busy: null,
};

class AppStore {
  boot = $state<Bootstrap | null>(null);
  settings = $state<Settings | null>(null);
  unlocked = $state(false);
  accounts = $state<Account[]>([]);
  folders = $state<Folder[]>([]);
  profiles = $state<ConfigProfile[]>([]);
  status = $state<RiotStatus>(emptyStatus);
  page = $state<Page>("accounts");
  toasts = $state<Toast[]>([]);
  performance = $state(false);
  /** Compte en cours de lancement (animation du bouton Jouer). */
  launching = $state<string | null>(null);
  /** Étapes franchies pendant le lancement en cours (pour l'overlay). */
  playSteps = $state<string[]>([]);
  /** Hors ligne par défaut dans le menu Jouer (mémorisé). */
  preferOffline = $state(false);
  loginHint = $state(false);
  /** Demande d'A2F en cours pendant une connexion (null = aucune). */
  mfa = $state<{ methods: string[]; email: string | null } | null>(null);
  /** Version Data Dragon (icônes publiques de champions / invocateur). */
  ddVersion = $state("");
  private toastSeq = 0;

  async init() {
    const boot = await api.bootstrap();
    this.boot = boot;
    this.settings = boot.settings;
    this.unlocked = boot.unlocked;
    try {
      this.preferOffline = localStorage.getItem("rs.offline") === "1";
    } catch {
      /* stockage indisponible */
    }
    // Premier lancement : langue détectée depuis le système.
    if (!boot.settings.language) {
      this.settings = await api.saveSettings({ ...boot.settings, language: detectLanguage() });
    }
    setLanguage(this.settings!.language || "en");
    await this.applyAppearance();
    await listen<RiotStatus>("riot-status", (e) => (this.status = e.payload));
    await listen("accounts-changed", () => this.refreshAccounts());
    await listen("vault-locked", () => {
      this.unlocked = false;
      this.accounts = [];
    });
    await listen("login-hint", () => {
      this.loginHint = true;
      this.toast("warning", t("login.staySignedIn"), 9000);
    });
    await listen<string>("session-expired", (e) => this.toast("info", t("play.sessionExpired", { name: e.payload }), 12000));
    await listen<{ methods: string[]; email: string | null }>("login-mfa", (e) => (this.mfa = e.payload));
    await listen<string>("play-step", (e) => (this.playSteps = [...this.playSteps, e.payload]));
    await listen("play-error", (e) => this.toast("error", errorText(e.payload)));
    this.status = await api.riotStatus();
    if (this.unlocked) await this.loadAll();
    latestVersion().then((v) => (this.ddVersion = v));

    // Activité utilisateur → repousse le verrouillage automatique (au plus une fois par 30 s).
    let last = 0;
    const touch = () => {
      const now = Date.now();
      if (now - last > 30_000 && this.unlocked) {
        last = now;
        api.touch().catch(() => {});
      }
    };
    window.addEventListener("pointerdown", touch, { passive: true });
    window.addEventListener("keydown", touch, { passive: true });
  }

  async applyAppearance() {
    const s = this.settings;
    if (!s) return;
    this.performance = s.performance === "performance" || (s.performance === "auto" && (await autoPerformance()));
    await applyTheme(s.theme, this.performance);
    try {
      await getCurrentWebview().setZoom(s.zoom / 100);
    } catch {
      document.documentElement.style.setProperty("zoom", String(s.zoom / 100));
    }
  }

  async saveSettings(patch: Partial<Settings>) {
    if (!this.settings) return;
    const next = { ...this.settings, ...patch };
    this.settings = await api.saveSettings(next);
    if (patch.language !== undefined) setLanguage(this.settings.language);
    if (patch.theme !== undefined || patch.performance !== undefined || patch.zoom !== undefined) await this.applyAppearance();
  }

  async loadAll() {
    await Promise.all([this.refreshAccounts(), this.refreshProfiles()]);
  }

  async refreshAccounts() {
    if (!this.unlocked) return;
    try {
      const [accounts, folders] = await Promise.all([api.listAccounts(), api.listFolders()]);
      this.accounts = accounts;
      this.folders = folders;
    } catch (e) {
      const err = e as { code?: string };
      if (err.code === "vault_locked") this.unlocked = false;
    }
  }

  async refreshProfiles() {
    this.profiles = await api.listProfiles().catch(() => []);
  }

  onUnlocked() {
    this.unlocked = true;
    this.loadAll();
  }

  async lock() {
    await api.lock();
    this.unlocked = false;
    this.accounts = [];
  }

  toast(kind: Toast["kind"], text: string, ms = 4200) {
    const id = ++this.toastSeq;
    this.toasts = [...this.toasts, { id, kind, text }];
    setTimeout(() => (this.toasts = this.toasts.filter((x) => x.id !== id)), ms);
  }

  setPreferOffline(v: boolean) {
    this.preferOffline = v;
    try {
      localStorage.setItem("rs.offline", v ? "1" : "0");
    } catch {
      /* ignoré */
    }
  }

  /** Lance un jeu sur un compte (bascule si nécessaire). */
  async play(account: Account, game: Game, offline: boolean) {
    if (this.launching) return;
    this.launching = account.id;
    this.playSteps = [];
    try {
      const notice = offline ? { title: t("offline.notifyTitle"), body: t("offline.notifyBody", { game: t(`games.${game}`) }) } : null;
      const r = await api.play(account.id, game, offline, notice);
      this.toast("success", r.switched ? t("play.switched", { name: account.label }) : t("play.launched", { game: t(`games.${game}`) }));
    } catch (e) {
      this.toast("error", errorText(e), 6000);
    } finally {
      this.launching = null;
      this.playSteps = [];
      this.refreshAccounts();
    }
  }

  get activeAccount(): Account | undefined {
    return this.accounts.find((a) => a.id === this.status.activeAccountId);
  }
}

export const app = new AppStore();
