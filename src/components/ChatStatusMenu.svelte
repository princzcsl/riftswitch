<script lang="ts">
  // Statut affiché aux amis quand le jeu passe par le mode fantôme.
  import { api } from "../lib/api";
  import { app } from "../lib/app.svelte";
  import { errorText, t } from "../lib/i18n.svelte";
  import ContextMenu from "./ContextMenu.svelte";
  import Icon from "./Icon.svelte";

  let {
    open = $bindable(false),
    x = 0,
    y = 0,
    fromBottom = false,
  }: { open?: boolean; x?: number; y?: number; fromBottom?: boolean } = $props();

  const STATUSES = ["online", "away", "mobile", "offline"] as const;

  async function pick(s: (typeof STATUSES)[number]) {
    open = false;
    try {
      if (await api.setChatStatus(s)) app.status = { ...app.status, chatStatus: s, offline: s !== "online" };
    } catch (e) {
      app.toast("error", errorText(e));
    }
  }
</script>

<ContextMenu bind:open {x} {y} {fromBottom}>
  {#each STATUSES as s}
    <button class="menu-item" onclick={() => pick(s)} title={s === "away" ? t("chatStatus.awayHint") : undefined}>
      <span class="status-dot {s}"></span>
      <span class="grow">{t(`chatStatus.${s}`)}</span>
      {#if app.status.chatStatus === s}<Icon name="check" size={14} />{/if}
    </button>
  {/each}
  <div class="sep"></div>
  <button
    class="menu-item muted-item"
    onclick={() => {
      open = false;
      app.setGhost(false);
    }}
  >
    <Icon name="ghost" size={14} />
    <span class="grow">{t("chatStatus.disable")}</span>
  </button>
</ContextMenu>

<style>
  .sep {
    height: 1px;
    margin: 6px 4px;
    background: var(--border);
  }
  .muted-item {
    color: var(--text-2) !important;
  }
  :global(.status-dot) {
    width: 9px;
    height: 9px;
    border-radius: 50%;
    flex: none;
  }
  :global(.status-dot.online) {
    background: var(--success);
  }
  :global(.status-dot.away) {
    background: var(--warning);
  }
  :global(.status-dot.mobile) {
    background: var(--accent-2);
  }
  :global(.status-dot.offline) {
    border: 2px solid var(--text-3);
  }
</style>
