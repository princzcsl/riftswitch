<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { api } from "../lib/api";
  import { app } from "../lib/app.svelte";
  import { t } from "../lib/i18n.svelte";
  import { displayRiotId } from "../lib/format";
  import Icon from "./Icon.svelte";

  const win = getCurrentWindow();
  let maximized = $state(false);

  $effect(() => {
    win.isMaximized().then((m) => (maximized = m));
    const un = win.onResized(async () => (maximized = await win.isMaximized()));
    return () => {
      un.then((f) => f());
    };
  });

  const status = $derived(app.status);
  const active = $derived(app.activeAccount);
  const streamer = $derived(app.settings?.streamerMode ?? false);
  const label = $derived.by(() => {
    if (status.busy === "switching") return t("status.switching");
    if (status.busy === "waiting_login") return t("status.waitingLogin");
    if (status.procs.lolGame) return t("status.inGame", { game: "League of Legends" });
    if (status.procs.valGame) return t("status.inGame", { game: "VALORANT" });
    if (!status.procs.riotClient) return status.clientInstalled ? t("status.clientClosed") : t("status.clientMissing");
    if (active) return t("status.connectedAs", { name: streamer ? active.label : `${active.label} · ${displayRiotId(active, false)}` });
    if (status.identity) return t("status.connectedUnknown", { id: streamer ? "•••" : `${status.identity.gameName}#${status.identity.tagLine}` });
    return t("status.clientOpen");
  });
  const tone = $derived(
    status.busy ? "busy" : status.procs.lolGame || status.procs.valGame ? "game" : status.procs.riotClient ? "on" : "off",
  );
</script>

<header class="titlebar" data-tauri-drag-region>
  <div class="brand" data-tauri-drag-region>
    <img src="/logo.svg" alt="" width="20" height="20" />
    <span data-tauri-drag-region>RiftSwitch</span>
  </div>

  <div class="status {tone}" data-tauri-drag-region>
    {#if status.busy}
      <svg class="spin" width="12" height="12" viewBox="0 0 24 24"><path d="M12 3a9 9 0 1 0 9 9" stroke="currentColor" stroke-width="3" fill="none" stroke-linecap="round" /></svg>
    {:else}
      <span class="dot" class:pulse={tone === "game"}></span>
    {/if}
    <span class="ellipsis" data-tauri-drag-region>{label}</span>
    {#if status.busy === "waiting_login"}
      <button class="chip cancel" onclick={() => api.cancelLogin()}>{t("common.cancel")}</button>
    {/if}
    {#if status.offline}
      <span class="chip warning" title={t("offline.active")}><Icon name="ghost" size={12} /> {t("offline.short")}</span>
    {/if}
  </div>

  <div class="actions">
    <button
      class="tb-btn"
      class:active={streamer}
      title={t("streamer.toggle")}
      onclick={() => app.saveSettings({ streamerMode: !streamer })}
    >
      <Icon name={streamer ? "eyeoff" : "eye"} size={16} />
    </button>
    <button class="tb-btn" title={t("window.minimize")} onclick={() => win.minimize()}><Icon name="min" size={16} /></button>
    <button class="tb-btn" title={t("window.maximize")} onclick={() => win.toggleMaximize()}>
      <Icon name={maximized ? "restore" : "max"} size={14} />
    </button>
    <button class="tb-btn close" title={t("window.close")} onclick={() => api.closeWindow()}><Icon name="x" size={16} /></button>
  </div>
</header>

<style>
  .titlebar {
    height: var(--titlebar-h);
    display: grid;
    grid-template-columns: 1fr auto 1fr;
    align-items: center;
    padding-left: 14px;
    position: relative;
    z-index: 30;
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 9px;
    font-weight: 700;
    letter-spacing: 0.01em;
    font-size: 13px;
  }
  .status {
    display: flex;
    align-items: center;
    gap: 8px;
    max-width: 52vw;
    padding: 4px 12px;
    border-radius: 999px;
    font-size: 12px;
    font-weight: 600;
    color: var(--text-2);
    background: var(--surface);
    border: 1px solid var(--border);
  }
  .status.on .dot {
    color: var(--success);
  }
  .status.off .dot {
    color: var(--text-3);
  }
  .status.game .dot {
    color: var(--accent-2);
  }
  .status.busy {
    color: var(--accent);
  }
  .cancel {
    cursor: pointer;
  }
  .actions {
    justify-self: end;
    display: flex;
    height: 100%;
  }
  .tb-btn {
    width: 46px;
    height: 100%;
    border: none;
    background: transparent;
    color: var(--text-2);
    display: grid;
    place-items: center;
    cursor: pointer;
    transition: background 0.12s;
  }
  .tb-btn:hover {
    background: var(--surface-hover);
    color: var(--text);
  }
  .tb-btn.active {
    color: var(--warning);
  }
  .tb-btn.close:hover {
    background: #e81123;
    color: #fff;
  }
</style>
