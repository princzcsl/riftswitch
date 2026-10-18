<script lang="ts">
  // Premier lancement guidé : langue + thème → sécurité → confidentialité → premier compte.
  import { open as openDialog } from "@tauri-apps/plugin-dialog";
  import { api } from "../lib/api";
  import { app } from "../lib/app.svelte";
  import { LANGUAGES, errorText, i18n, setLanguage, t } from "../lib/i18n.svelte";
  import { THEMES } from "../lib/themes";
  import Icon from "../components/Icon.svelte";
  import LoginWait from "../components/LoginWait.svelte";
  import Toggle from "../components/Toggle.svelte";

  let { initial = 0 }: { initial?: number } = $props();
  // svelte-ignore state_referenced_locally
  let step = $state(initial);
  let mode = $state<"password" | "os">("password");
  let pw = $state("");
  let pw2 = $state("");
  let ack = $state(false);
  let telemetry = $state(false);
  let busy = $state(false);
  let error = $state("");
  let loginOpen = $state(false);
  let importPass = $state("");

  const portable = $derived(app.boot?.portable ?? false);
  const strength = $derived(Math.min(4, Math.floor(pw.length / 4) + (/[A-Z]/.test(pw) ? 1 : 0) + (/[^a-zA-Z0-9]/.test(pw) ? 1 : 0) - 1));
  const steps = ["welcome", "security", "privacy", "account"];

  async function chooseLang(code: string) {
    setLanguage(code);
    await app.saveSettings({ language: code });
  }

  async function createVault() {
    error = "";
    if (mode === "password") {
      if (pw.length < 8) return (error = t("errors.password_too_short"));
      if (pw !== pw2) return (error = t("setup.mismatch"));
    } else if (!ack) return (error = t("setup.ackRequired"));
    busy = true;
    try {
      await api.setupVault(mode === "password" ? pw : null);
      pw = pw2 = "";
      app.boot = await api.bootstrap();
      app.onUnlocked();
      step = 2;
    } catch (e) {
      error = errorText(e);
    } finally {
      busy = false;
    }
  }

  async function finishPrivacy() {
    await app.saveSettings({ telemetry });
    step = 3;
  }

  async function finish() {
    await app.saveSettings({ setupDone: true });
    app.boot = await api.bootstrap();
  }

  async function importBackup() {
    const path = await openDialog({ filters: [{ name: "RiftSwitch", extensions: ["rsbackup", "rsv", "json"] }] });
    if (!path || Array.isArray(path)) return;
    try {
      const n = await api.importBackup(path, importPass);
      app.toast("success", t("settings.imported", { n }));
      await app.refreshAccounts();
    } catch (e) {
      app.toast("error", errorText(e));
    }
  }
</script>

<div class="setup">
  <div class="panel glass-strong fade-in">
    <div class="progress">
      {#each steps as s, i}<span class:done={i <= step}></span>{/each}
    </div>

    {#if step === 0}
      <div class="hero">
        <img src="/logo.svg" alt="" width="96" height="96" />
        <h1>{t("setup.welcome")}</h1>
        <p class="muted">{t("setup.tagline")}</p>
      </div>
      <div class="field">
        <span class="label">{t("settings.language")}</span>
        <div class="langs">
          {#each LANGUAGES as l}
            <button class="lang" class:on={i18n.lang === l.code} onclick={() => chooseLang(l.code)}>
              {l.name}{#if !l.complete}<small class="beta">β</small>{/if}
            </button>
          {/each}
        </div>
      </div>
      <div class="field" style="margin-top:16px">
        <span class="label">{t("settings.theme")}</span>
        <div class="themes">
          {#each THEMES.filter((th) => th.cost === 0) as th}
            <button
              class="theme"
              class:on={app.settings?.theme === th.id}
              style="--a:{th.tokens.accent};--b:{th.tokens['accent-2']};--bg:{th.tokens['bg-solid']};--tx:{th.tokens.text}"
              onclick={() => app.saveSettings({ theme: th.id })}
            >
              <span class="sw"></span><span>{th.name}</span>
            </button>
          {/each}
        </div>
      </div>
      <div class="nav"><span></span><button class="btn primary" onclick={() => (step = 1)}>{t("common.next")}<Icon name="chevronRight" size={16} /></button></div>
    {:else if step === 1}
      <h2>{t("setup.securityTitle")}</h2>
      <p class="muted">{t("setup.securityText")}</p>
      <div class="options">
        <button class="opt" class:on={mode === "password"} onclick={() => (mode = "password")}>
          <Icon name="lock" size={22} />
          <div><strong>{t("setup.withPassword")}</strong> <span class="chip success">{t("setup.recommended")}</span><p class="small muted">{t("setup.withPasswordText")}</p></div>
        </button>
        <button class="opt" class:on={mode === "os"} onclick={() => (mode = "os")} disabled={portable}>
          <Icon name="unlock" size={22} />
          <div><strong>{t("setup.withoutPassword")}</strong><p class="small muted">{portable ? t("setup.portableNeedsPassword") : t("setup.withoutPasswordText")}</p></div>
        </button>
      </div>
      {#if mode === "password"}
        <div class="grid2">
          <div class="field">
            <label for="p1">{t("setup.masterPassword")}</label>
            <input id="p1" class="input" type="password" autocomplete="new-password" bind:value={pw} />
            <div class="meter"><span style="width:{Math.max(0, strength) * 25}%" class="s{Math.max(0, strength)}"></span></div>
          </div>
          <div class="field">
            <label for="p2">{t("setup.confirmPassword")}</label>
            <input id="p2" class="input" type="password" autocomplete="new-password" bind:value={pw2} onkeydown={(e) => e.key === "Enter" && createVault()} />
          </div>
        </div>
        <p class="faint small">{t("setup.passwordNote")}</p>
      {:else}
        <div class="warn">
          <Icon name="alert" size={18} />
          <div>
            <p>{t("setup.osWarning")}</p>
            <label class="row small"><input type="checkbox" bind:checked={ack} />{t("setup.osAck")}</label>
          </div>
        </div>
      {/if}
      {#if error}<p class="error">{error}</p>{/if}
      <div class="nav">
        <button class="btn ghost" onclick={() => (step = 0)}>{t("common.back")}</button>
        <button class="btn primary" onclick={createVault} disabled={busy}>
          {#if busy}<svg class="spin" width="14" height="14" viewBox="0 0 24 24"><path d="M12 3a9 9 0 1 0 9 9" stroke="currentColor" stroke-width="3" fill="none" /></svg>{/if}
          {t("setup.createVault")}
        </button>
      </div>
    {:else if step === 2}
      <h2>{t("setup.privacyTitle")}</h2>
      <p class="muted">{t("setup.privacyText")}</p>
      <div class="opt static">
        <Toggle bind:checked={telemetry} />
        <div>
          <strong>{t("settings.telemetry")}</strong>
          <p class="small muted">{t("settings.telemetryText")}</p>
        </div>
      </div>
      <ul class="small muted bullets">
        <li>{t("setup.privacy1")}</li>
        <li>{t("setup.privacy2")}</li>
        <li>{t("setup.privacy3")}</li>
      </ul>
      <div class="nav"><span></span><button class="btn primary" onclick={finishPrivacy}>{t("common.next")}<Icon name="chevronRight" size={16} /></button></div>
    {:else}
      <h2>{t("setup.accountTitle")}</h2>
      <p class="muted">{t("setup.accountText")}</p>
      {#if app.accounts.length}
        <div class="done"><Icon name="check" size={18} />{t("setup.accountsReady", { n: app.accounts.length })}</div>
      {/if}
      <div class="options">
        <button class="opt" onclick={() => (loginOpen = true)}>
          <Icon name="login" size={22} />
          <div><strong>{t("accounts.addLogin")}</strong><p class="small muted">{t("accounts.addLoginHint")}</p></div>
        </button>
        <button
          class="opt"
          onclick={async () => {
            try {
              await api.captureCurrent(null);
              await app.refreshAccounts();
              app.toast("success", t("login.added"));
            } catch (e) {
              app.toast("error", errorText(e), 6000);
            }
          }}
        >
          <Icon name="download" size={22} />
          <div><strong>{t("accounts.addCurrent")}</strong><p class="small muted">{t("accounts.addCurrentHint")}</p></div>
        </button>
        <div class="opt static">
          <Icon name="upload" size={22} />
          <div class="grow">
            <strong>{t("setup.importBackup")}</strong>
            <div class="row" style="margin-top:6px">
              <input class="input grow" type="password" placeholder={t("settings.passphrase")} bind:value={importPass} />
              <button class="btn sm" onclick={importBackup} disabled={importPass.length < 8}>{t("common.choose")}</button>
            </div>
          </div>
        </div>
      </div>
      <div class="nav">
        <span></span>
        <button class="btn primary lg" onclick={finish}>{app.accounts.length ? t("setup.start") : t("setup.later")}</button>
      </div>
    {/if}
  </div>
</div>

<LoginWait bind:open={loginOpen} />

<style>
  .setup {
    height: 100%;
    display: grid;
    place-items: center;
    padding: 20px;
    overflow-y: auto;
  }
  .panel {
    width: min(640px, 100%);
    padding: 28px 32px;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }
  .progress {
    display: flex;
    gap: 6px;
  }
  .progress span {
    flex: 1;
    height: 4px;
    border-radius: 4px;
    background: var(--border);
  }
  .progress span.done {
    background: linear-gradient(90deg, var(--accent), var(--accent-2));
  }
  .hero {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: 6px;
    margin: 6px 0 10px;
  }
  .hero img {
    filter: drop-shadow(0 12px 30px var(--accent-soft));
  }
  .langs {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .lang {
    padding: 6px 12px;
    border-radius: var(--radius-pill);
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--text-2);
    cursor: pointer;
    font-weight: 600;
    font-size: 12.5px;
  }
  .beta {
    margin-left: 5px;
    color: var(--text-3);
    font-size: 10px;
  }
  .lang.on {
    background: var(--accent-soft);
    color: var(--text);
    border-color: var(--accent);
  }
  .themes {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
    gap: 8px;
  }
  .theme {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px;
    border-radius: var(--radius-sm);
    border: 1.5px solid var(--border);
    background: var(--bg);
    color: var(--tx);
    cursor: pointer;
    font-weight: 600;
    font-size: 12.5px;
  }
  .theme.on {
    border-color: var(--a);
  }
  .sw {
    width: 26px;
    height: 26px;
    border-radius: 50%;
    background: linear-gradient(135deg, var(--a), var(--b));
    flex: none;
  }
  .options {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .opt {
    display: flex;
    gap: 14px;
    align-items: flex-start;
    padding: 14px;
    text-align: start;
    border-radius: var(--radius-md);
    border: 1.5px solid var(--border);
    background: var(--surface);
    color: var(--text);
    cursor: pointer;
  }
  .opt.static {
    cursor: default;
  }
  .opt:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .opt.on {
    border-color: var(--accent);
    background: var(--accent-soft);
  }
  .opt p {
    margin: 4px 0 0;
  }
  .opt :global(svg) {
    color: var(--accent);
    margin-top: 2px;
  }
  .grid2 {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
  }
  .meter {
    height: 4px;
    border-radius: 4px;
    background: var(--border);
    overflow: hidden;
  }
  .meter span {
    display: block;
    height: 100%;
    transition: width 0.2s;
  }
  .s0,
  .s1 {
    background: var(--danger);
  }
  .s2 {
    background: var(--warning);
  }
  .s3,
  .s4 {
    background: var(--success);
  }
  .warn {
    display: flex;
    gap: 12px;
    padding: 12px 14px;
    border-radius: var(--radius-sm);
    background: color-mix(in srgb, var(--warning) 14%, transparent);
    color: var(--text);
  }
  .warn :global(svg) {
    color: var(--warning);
    flex: none;
  }
  .warn p {
    margin: 0 0 8px;
  }
  .error {
    color: var(--danger);
    margin: 0;
  }
  .bullets {
    margin: 0;
    padding-left: 18px;
    line-height: 1.7;
  }
  .done {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--success);
    font-weight: 600;
  }
  .nav {
    display: flex;
    justify-content: space-between;
    margin-top: 8px;
  }
</style>
