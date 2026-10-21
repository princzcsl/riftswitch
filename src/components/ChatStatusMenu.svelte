<script lang="ts">
  // Statut affiché aux amis quand le jeu passe par le mode fantôme.
  import { api } from "../lib/api";
  import { app } from "../lib/app.svelte";
  import { errorText, t } from "../lib/i18n.svelte";
  import ContextMenu from "./ContextMenu.svelte";
  import Icon from "./Icon.svelte";

  let { open = $bindable(false), x = 0, y = 0 }: { open?: boolean; x?: number; y?: number } = $props();

  const STATUSES = ["online", "away", "busy", "mobile", "offline"] as const;

  async function pick(s: (typeof STATUSES)[number]) {
    open = false;
    try {
      if (await api.setChatStatus(s)) app.status = { ...app.status, chatStatus: s, offline: s !== "online" };
    } catch (e) {
      app.toast("error", errorText(e));
    }
  }
</script>

<ContextMenu bind:open {x} {y}>
  {#each STATUSES as s}
    <button class="menu-item" onclick={() => pick(s)}>
      <span class="status-dot {s}"></span>
      <span class="grow">{t(`chatStatus.${s}`)}</span>
      {#if app.status.chatStatus === s}<Icon name="check" size={14} />{/if}
    </button>
  {/each}
</ContextMenu>

<style>
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
  :global(.status-dot.busy) {
    background: var(--danger);
  }
  :global(.status-dot.mobile) {
    background: var(--accent-2);
  }
  :global(.status-dot.offline) {
    border: 2px solid var(--text-3);
  }
</style>
