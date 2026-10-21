<script lang="ts">
  import { app, type Page } from "../lib/app.svelte";
  import { t } from "../lib/i18n.svelte";
  import ChatStatusMenu from "./ChatStatusMenu.svelte";
  import Icon from "./Icon.svelte";

  let statusMenu = $state({ open: false, x: 0, y: 0 });

  const items: { page: Page; icon: string }[] = [
    { page: "accounts", icon: "users" },
    { page: "tonight", icon: "moon" },
    { page: "configs", icon: "keyboard" },
    { page: "runes", icon: "hex" },
    { page: "shops", icon: "shop" },
    { page: "patches", icon: "scroll" },
    { page: "esport", icon: "trophy" },
    { page: "achievements", icon: "award" },
  ];

  let { badges = {} }: { badges?: Partial<Record<Page, boolean>> } = $props();
  const lockable = $derived(app.boot?.vaultMode === "password");
</script>

<nav class="sidebar">
  {#each items as item}
    <button class="nav" class:active={app.page === item.page} onclick={() => (app.page = item.page)} title={t(`nav.${item.page}`)}>
      <span class="ic"><Icon name={item.icon} size={20} />{#if badges[item.page]}<i class="badge"></i>{/if}</span>
      <span class="lbl">{t(`nav.${item.page}`)}</span>
    </button>
  {/each}
  <div class="spacer"></div>
  {#if app.status.proxyRunning}
    <button
      class="nav"
      class:ghost={app.status.offline}
      title={t("chatStatus.title")}
      onclick={(e) => ((statusMenu = { open: true, x: e.clientX + 12, y: e.clientY - 220 }))}
    >
      <span class="ic"><Icon name="ghost" size={20} /></span>
      <span class="lbl">{t(`chatStatus.${app.status.chatStatus ?? "online"}`)}</span>
    </button>
    <ChatStatusMenu bind:open={statusMenu.open} x={statusMenu.x} y={statusMenu.y} />
  {/if}
  {#if lockable}
    <button class="nav" onclick={() => app.lock()} title={t("nav.lock")}>
      <span class="ic"><Icon name="lock" size={20} /></span>
      <span class="lbl">{t("nav.lock")}</span>
    </button>
  {/if}
  <button class="nav" class:active={app.page === "settings"} onclick={() => (app.page = "settings")} title={t("nav.settings")}>
    <span class="ic"><Icon name="settings" size={20} /></span>
    <span class="lbl">{t("nav.settings")}</span>
  </button>
</nav>

<style>
  .sidebar {
    width: var(--sidebar-w);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    padding: 10px 8px 14px;
    border-right: 1px solid var(--border);
    overflow-y: auto;
    overflow-x: hidden;
  }
  .nav {
    width: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    padding: 9px 2px 7px;
    border: none;
    border-radius: var(--radius-md);
    background: transparent;
    color: var(--text-3);
    cursor: pointer;
    position: relative;
    transition:
      color 0.15s,
      background 0.15s;
  }
  .nav:hover {
    color: var(--text);
    background: var(--surface-hover);
  }
  .nav.active {
    color: var(--text);
    background: var(--accent-soft);
  }
  .nav.active::before {
    content: "";
    position: absolute;
    left: -8px;
    top: 22%;
    bottom: 22%;
    width: 3px;
    border-radius: 3px;
    background: linear-gradient(var(--accent), var(--accent-2));
  }
  .nav.active .ic {
    color: var(--accent);
  }
  /* Fantôme : couleur du texte du thème (blanc en sombre, noir en clair). */
  .nav.ghost {
    color: var(--text);
  }
  .ic {
    position: relative;
    display: grid;
  }
  .badge {
    position: absolute;
    top: -2px;
    right: -4px;
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--danger);
    box-shadow: 0 0 0 2px var(--bg-solid);
  }
  .lbl {
    font-size: 10.5px;
    font-weight: 600;
    max-width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
