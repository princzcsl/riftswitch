<script lang="ts">
  import { open as openDialog, save as saveDialog } from "@tauri-apps/plugin-dialog";
  import { api } from "../lib/api";
  import { app } from "../lib/app.svelte";
  import { LANGUAGES, errorText, t } from "../lib/i18n.svelte";
  import { THEMES } from "../lib/themes";
  import { updater } from "../lib/updater.svelte";
  import { computeAchievements, shards } from "../lib/achievements";
  import Icon from "../components/Icon.svelte";
  import Modal from "../components/Modal.svelte";
  import Toggle from "../components/Toggle.svelte";

  const s = $derived(app.settings!);
  const balance = $derived(shards(computeAchievements(app.accounts, app.folders, app.profiles)) + (app.settings?.bonusShards ?? 0));

  let section = $state("general");
  const sections = ["general", "appearance", "security", "riot", "privacy", "backup", "advanced", "about"];

  // Mot de passe maître
  let pwOpen = $state(false);
  let newPw = $state("");
  let newPw2 = $state("");
  let removePw = $state(false);
  let pwBusy = $state(false);

  // Sauvegardes
  let passphrase = $state("");
  let backupBusy = $state(false);

  // Télémétrie
  let previewOpen = $state(false);
  let preview = $state<Record<string, unknown>>({});

  // Réinitialisation
  let resetOpen = $state(false);
  let resetStep = $state(0);

  async function checkNow() {
    try {
      if (!(await updater.check(true))) app.toast("success", t("update.upToDate"));
    } catch (e) {
      app.toast("error", `${t("update.failed")} ${errorText(e)}`);
    }
  }
  async function installUpdate() {
    try {
      updater.updating = true;
      await updater.install();
    } catch (e) {
      updater.updating = false;
      app.toast("error", `${t("update.failed")} ${errorText(e)}`);
    }
  }

  function set<K extends keyof typeof s>(key: K, value: (typeof s)[K]) {
    app.saveSettings({ [key]: value } as Partial<typeof s>);
  }

  async function changePassword() {
    if (!removePw && (newPw.length < 8 || newPw !== newPw2)) {
      app.toast("error", newPw.length < 8 ? t("errors.password_too_short") : t("setup.mismatch"));
      return;
    }
    pwBusy = true;
    try {
      await api.changeMasterPassword(removePw ? null : newPw);
      app.boot = await api.bootstrap();
      pwOpen = false;
      newPw = newPw2 = "";
      app.toast("success", t("settings.passwordChanged"));
    } catch (e) {
      app.toast("error", errorText(e));
    } finally {
      pwBusy = false;
    }
  }

  async function exportBackup() {
    if (passphrase.length < 8) return app.toast("error", t("errors.password_too_short"));
    const path = await saveDialog({ defaultPath: `riftswitch-${new Date().toISOString().slice(0, 10)}.rsbackup`, filters: [{ name: "RiftSwitch", extensions: ["rsbackup"] }] });
    if (!path) return;
    backupBusy = true;
    try {
      await api.exportBackup(path, passphrase);
      app.toast("success", t("settings.exported"));
      passphrase = "";
    } catch (e) {
      app.toast("error", errorText(e));
    } finally {
      backupBusy = false;
    }
  }

  async function importBackup() {
    const path = await openDialog({ filters: [{ name: "RiftSwitch", extensions: ["rsbackup", "json"] }] });
    if (!path || Array.isArray(path)) return;
    backupBusy = true;
    try {
      const n = await api.importBackup(path, passphrase);
      app.toast("success", t("settings.imported", { n }));
      passphrase = "";
      app.refreshAccounts();
    } catch (e) {
      app.toast("error", errorText(e));
    } finally {
      backupBusy = false;
    }
  }

  async function showPreview() {
    preview = await api.telemetryPreview();
    previewOpen = true;
  }

  async function reset() {
    if (resetStep === 0) return (resetStep = 1);
    await api.resetApp();
    location.reload();
  }
</script>

<div class="page">
  <div class="page-header"><h1>{t("nav.settings")}</h1></div>
  <div class="layout">
    <nav class="snav">
      {#each sections as k}
        <button class:on={section === k} onclick={() => (section = k)}>{t(`settings.s_${k}`)}</button>
      {/each}
    </nav>

    <div class="content glass fade-in">
      {#if section === "general"}
        <div class="setting">
          <div><strong>{t("settings.language")}</strong></div>
          <select class="input" value={s.language} onchange={(e) => set("language", e.currentTarget.value)}>
            {#each LANGUAGES as l}<option value={l.code}>{l.name}{l.complete ? "" : " (β)"}</option>{/each}
          </select>
        </div>
        <div class="setting">
          <div><strong>{t("settings.closeToTray")}</strong><p>{t("settings.closeToTrayText")}</p></div>
          <Toggle checked={s.closeToTray} onchange={(v) => set("closeToTray", v)} />
        </div>
        <div class="setting">
          <div><strong>{t("settings.startMinimized")}</strong><p>{t("settings.startMinimizedText")}</p></div>
          <Toggle checked={s.startMinimized} onchange={(v) => set("startMinimized", v)} />
        </div>
        <div class="setting">
          <div><strong>{t("settings.playLabels")}</strong><p>{t("settings.playLabelsText")}</p></div>
          <Toggle checked={s.playMenuLabels} onchange={(v) => set("playMenuLabels", v)} />
        </div>
      {:else if section === "appearance"}
        <span class="label">{t("settings.theme")}</span>
        <div class="themes">
          {#each THEMES as th}
            {@const locked = th.cost > balance}
            <button
              class="theme"
              class:on={s.theme === th.id}
              disabled={locked}
              style="--a:{th.tokens.accent};--b:{th.tokens['accent-2']};--bg:{th.tokens['bg-solid']};--tx:{th.tokens.text};--a1:{th.tokens['aurora-1']};--a2:{th.tokens['aurora-2']}"
              onclick={() => set("theme", th.id)}
            >
              <span class="preview"><span class="pbtn"></span><span class="pcard"></span></span>
              <span class="tname">{th.name}</span>
              {#if th.cost > 0}<span class="cost">{locked ? "🔒" : "✓"} {th.cost} ✦</span>{/if}
            </button>
          {/each}
        </div>
        <p class="faint small">{t("settings.themeShards", { n: balance })}</p>
        <div class="setting">
          <div><strong>{t("settings.performance")}</strong><p>{t("settings.performanceText")}</p></div>
          <select class="input" value={s.performance} onchange={(e) => set("performance", e.currentTarget.value as typeof s.performance)}>
            <option value="auto">{t("settings.perfAuto")}</option>
            <option value="quality">{t("settings.perfQuality")}</option>
            <option value="performance">{t("settings.perfLite")}</option>
          </select>
        </div>
        <div class="setting">
          <div><strong>{t("settings.zoom")}</strong><p>{t("settings.zoomText")}</p></div>
          <div class="row">
            <input type="range" min="80" max="150" step="5" value={s.zoom} onchange={(e) => set("zoom", Number(e.currentTarget.value))} />
            <span class="mono small" style="width:44px">{s.zoom}%</span>
          </div>
        </div>
      {:else if section === "security"}
        <div class="setting">
          <div>
            <strong>{t("settings.vaultMode")}</strong>
            <p>{app.boot?.vaultMode === "password" ? t("settings.vaultPassword") : t("settings.vaultOs")}</p>
          </div>
          <button class="btn" onclick={() => ((pwOpen = true), (removePw = false))}>
            {app.boot?.vaultMode === "password" ? t("settings.changePassword") : t("settings.addPassword")}
          </button>
        </div>
        <div class="setting">
          <div><strong>{t("settings.autoLock")}</strong><p>{t("settings.autoLockText")}</p></div>
          <select class="input" value={s.autoLockMinutes} onchange={(e) => set("autoLockMinutes", Number(e.currentTarget.value))} disabled={app.boot?.vaultMode !== "password"}>
            <option value={0}>{t("settings.never")}</option>
            {#each [5, 15, 30, 60, 240] as m}<option value={m}>{t("settings.afterMin", { n: m })}</option>{/each}
          </select>
        </div>
        <div class="setting">
          <div><strong>{t("settings.lockOnSleep")}</strong><p>{t("settings.lockOnSleepText")}</p></div>
          <Toggle checked={s.lockOnSleep} disabled={app.boot?.vaultMode !== "password"} onchange={(v) => set("lockOnSleep", v)} />
        </div>
        <div class="threat">
          <Icon name="shield" size={18} />
          <div>
            <strong>{t("settings.threatTitle")}</strong>
            <p>{t("settings.threatText")}</p>
          </div>
        </div>
      {:else if section === "riot"}
        <div class="setting">
          <div><strong>{t("settings.blockInGame")}</strong><p>{t("settings.blockInGameText")}</p></div>
          <Toggle checked={s.blockSwitchInGame} onchange={(v) => set("blockSwitchInGame", v)} />
        </div>
        <div class="setting">
          <div><strong>{t("settings.closeRiotClient")}</strong><p>{t("settings.closeRiotClientText")}</p></div>
          <Toggle checked={s.closeRiotClient} onchange={(v) => set("closeRiotClient", v)} />
        </div>
        <div class="setting">
          <div><strong>{t("settings.rememberGhost")}</strong><p>{t("settings.rememberGhostText")}</p></div>
          <Toggle checked={s.rememberGhost} onchange={(v) => set("rememberGhost", v)} />
        </div>
        <div class="setting">
          <div><strong>{t("settings.autoProfiles")}</strong><p>{t("settings.autoProfilesText")}</p></div>
          <Toggle checked={s.autoApplyProfiles} onchange={(v) => set("autoApplyProfiles", v)} />
        </div>
        <div class="setting col-setting">
          <div><strong>{t("settings.riotPath")}</strong><p>{app.status.clientInstalled ? t("settings.detected") : t("settings.notDetected")}</p></div>
          <input class="input" placeholder="C:\Riot Games\Riot Client\RiotClientServices.exe" value={s.riotClientPath} onchange={(e) => set("riotClientPath", e.currentTarget.value)} />
        </div>
        <div class="setting col-setting">
          <div><strong>{t("settings.lolPath")}</strong><p>{t("settings.pathAuto")}</p></div>
          <input class="input" placeholder="C:\Riot Games\League of Legends" value={s.lolPath} onchange={(e) => set("lolPath", e.currentTarget.value)} />
        </div>
        <div class="setting">
          <div><strong>{t("settings.discord")}</strong><p>{t("settings.discordText")}</p></div>
          <span class="chip">{t("common.soon")}</span>
        </div>
        <div class="setting">
          <div><strong>{t("settings.quitRiot")}</strong><p>{t("settings.quitRiotText")}</p></div>
          <button class="btn" onclick={() => api.quitRiot().then(() => app.toast("success", t("settings.riotClosed"))).catch((e) => app.toast("error", errorText(e)))}>
            <Icon name="power" size={15} />{t("settings.quitRiotBtn")}
          </button>
        </div>
      {:else if section === "privacy"}
        <div class="setting">
          <div><strong>{t("streamer.title")}</strong><p>{t("streamer.text")}</p></div>
          <Toggle checked={s.streamerMode} onchange={(v) => set("streamerMode", v)} />
        </div>
        <div class="setting">
          <div><strong>{t("settings.telemetry")}</strong><p>{t("settings.telemetryText")}</p></div>
          <div class="row">
            <button class="btn sm ghost" onclick={showPreview}>{t("settings.seeWhat")}</button>
            <Toggle checked={s.telemetry} onchange={(v) => set("telemetry", v)} />
          </div>
        </div>
        <div class="section-title">{t("settings.riskyFeatures")}</div>
        <div class="setting">
          <div><strong>{t("settings.valStore")}</strong> <span class="risk medium">● {t("risk.medium")}</span><p>{t("settings.valStoreText")}</p></div>
          <Toggle checked={s.valorantStoreOptIn} onchange={(v) => set("valorantStoreOptIn", v)} />
        </div>
        <div class="setting">
          <div><strong>{t("settings.thirdParty")}</strong> <span class="risk low">● {t("risk.low")}</span><p>{t("settings.thirdPartyText")}</p></div>
          <Toggle checked={s.thirdPartyRanksOptIn} onchange={(v) => set("thirdPartyRanksOptIn", v)} />
        </div>
      {:else if section === "backup"}
        <p class="muted">{t("settings.backupText")}</p>
        <div class="field" style="max-width:360px">
          <label for="pp">{t("settings.passphrase")}</label>
          <input id="pp" class="input" type="password" autocomplete="new-password" bind:value={passphrase} />
        </div>
        <div class="row" style="margin-top:12px">
          <button class="btn primary" onclick={exportBackup} disabled={backupBusy || passphrase.length < 8}><Icon name="download" size={15} />{t("settings.export")}</button>
          <button class="btn" onclick={importBackup} disabled={backupBusy || passphrase.length < 8}><Icon name="upload" size={15} />{t("settings.import")}</button>
        </div>
      {:else if section === "advanced"}
        <div class="setting">
          <div><strong>{t("settings.dataDir")}</strong><p class="mono small">{app.boot?.dataDir}</p></div>
          <button class="btn" onclick={() => api.openDataDir()}><Icon name="folder" size={15} />{t("settings.open")}</button>
        </div>
        <div class="setting danger-zone">
          <div><strong>{t("settings.reset")}</strong><p>{t("settings.resetText")}</p></div>
          <button class="btn danger" onclick={() => ((resetOpen = true), (resetStep = 0))}>{t("settings.reset")}</button>
        </div>
      {:else}
        <div class="about">
          <img src="/logo.svg" alt="" width="72" height="72" />
          <div>
            <h2>RiftSwitch</h2>
            <p class="muted">v{app.boot?.version}{app.boot?.portable ? " · portable" : ""}</p>
          </div>
        </div>
        <div class="setting">
          <div><strong>{t("update.auto")}</strong><p>{t("update.autoText")}</p></div>
          <Toggle checked={s.checkUpdates} onchange={(v) => set("checkUpdates", v)} />
        </div>
        <div class="setting">
          <div>
            <strong>{t("update.check")}</strong>
            <p>{updater.available ? t("update.available", { v: updater.available.version }) : t("update.current", { v: app.boot?.version ?? "" })}</p>
          </div>
          {#if updater.available}
            <button class="btn primary" onclick={installUpdate} disabled={updater.progress !== null}><Icon name="download" size={15} />{t("update.install")}</button>
          {:else}
            <button class="btn" onclick={checkNow} disabled={updater.checking}><Icon name="refresh" size={15} />{t("update.checkNow")}</button>
          {/if}
        </div>
        <p class="small muted">{t("about.legal")}</p>
        <p class="small muted">{t("about.vanguard")}</p>
      {/if}
    </div>
  </div>
</div>

<Modal bind:open={pwOpen} title={app.boot?.vaultMode === "password" ? t("settings.changePassword") : t("settings.addPassword")} width={460}>
  <div class="col">
    {#if app.boot?.vaultMode === "password" && !app.boot?.portable}
      <label class="row small"><input type="checkbox" bind:checked={removePw} />{t("settings.removePassword")}</label>
    {/if}
    {#if removePw}
      <p class="small" style="color:var(--warning)">{t("setup.osWarning")}</p>
    {:else}
      <input class="input" type="password" autocomplete="new-password" placeholder={t("setup.masterPassword")} bind:value={newPw} />
      <input class="input" type="password" autocomplete="new-password" placeholder={t("setup.confirmPassword")} bind:value={newPw2} />
    {/if}
  </div>
  {#snippet footer()}
    <button class="btn" onclick={() => (pwOpen = false)}>{t("common.cancel")}</button>
    <button class="btn primary" onclick={changePassword} disabled={pwBusy}>{t("common.save")}</button>
  {/snippet}
</Modal>

<Modal bind:open={previewOpen} title={t("settings.seeWhat")} width={520}>
  <p class="small muted">{t("settings.previewText")}</p>
  <pre class="mono small pre">{JSON.stringify(preview, null, 2)}</pre>
</Modal>

<Modal bind:open={resetOpen} title={t("settings.resetTitle")} width={440}>
  <p>{t("settings.resetText")}</p>
  {#if resetStep === 1}<p style="color:var(--danger)">{t("settings.resetConfirm2")}</p>{/if}
  {#snippet footer()}
    <button class="btn" onclick={() => (resetOpen = false)}>{t("common.cancel")}</button>
    <button class="btn danger solid" onclick={reset}>{resetStep === 0 ? t("settings.reset") : t("settings.resetFinal")}</button>
  {/snippet}
</Modal>

<style>
  .layout {
    display: grid;
    grid-template-columns: 200px 1fr;
    gap: 18px;
    align-items: start;
  }
  .snav {
    display: flex;
    flex-direction: column;
    gap: 2px;
    position: sticky;
    top: 0;
  }
  .snav button {
    text-align: start;
    padding: 9px 12px;
    border: none;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
    font-weight: 600;
  }
  .snav button:hover {
    background: var(--surface-hover);
  }
  .snav button.on {
    background: var(--accent-soft);
    color: var(--text);
  }
  .content {
    padding: 8px 22px 22px;
    display: flex;
    flex-direction: column;
  }
  .setting {
    display: flex;
    align-items: center;
    gap: 20px;
    padding: 16px 0;
    border-bottom: 1px solid var(--border);
  }
  .setting > div:first-child {
    flex: 1;
  }
  .setting p {
    margin: 3px 0 0;
    color: var(--text-2);
    font-size: 12.5px;
  }
  .setting select.input {
    width: 220px;
  }
  .col-setting {
    flex-direction: column;
    align-items: stretch;
    gap: 8px;
  }
  .themes {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: 10px;
    margin: 10px 0 6px;
  }
  .theme {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 8px;
    border-radius: var(--radius-md);
    border: 2px solid var(--border);
    background: transparent;
    cursor: pointer;
    color: var(--text);
    text-align: start;
  }
  .theme:disabled {
    opacity: 0.55;
    cursor: not-allowed;
  }
  .theme.on {
    border-color: var(--accent);
  }
  .preview {
    position: relative;
    height: 70px;
    border-radius: 8px;
    overflow: hidden;
    background:
      radial-gradient(circle at 15% 10%, var(--a1), transparent 60%),
      radial-gradient(circle at 90% 90%, var(--a2), transparent 60%),
      var(--bg);
  }
  .pcard {
    position: absolute;
    left: 10px;
    top: 10px;
    width: 60%;
    height: 50px;
    border-radius: 6px;
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid rgba(255, 255, 255, 0.15);
  }
  .pbtn {
    position: absolute;
    right: 10px;
    bottom: 10px;
    width: 36px;
    height: 14px;
    border-radius: 4px;
    background: linear-gradient(135deg, var(--a), var(--b));
    z-index: 1;
  }
  .tname {
    font-weight: 650;
    font-size: 12.5px;
  }
  .cost {
    font-size: 11px;
    color: var(--text-3);
  }
  .threat {
    display: flex;
    gap: 12px;
    margin-top: 16px;
    padding: 14px;
    border-radius: var(--radius-sm);
    background: var(--accent-soft);
  }
  .threat :global(svg) {
    color: var(--accent);
    flex: none;
  }
  .threat p {
    margin: 4px 0 0;
    color: var(--text-2);
    font-size: 12.5px;
    line-height: 1.5;
  }
  .danger-zone strong {
    color: var(--danger);
  }
  .about {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 16px 0;
  }
  .pre {
    padding: 12px;
    border-radius: var(--radius-sm);
    background: var(--surface);
    border: 1px solid var(--border);
    user-select: text;
    white-space: pre-wrap;
  }
  input[type="range"] {
    accent-color: var(--accent);
  }
</style>
