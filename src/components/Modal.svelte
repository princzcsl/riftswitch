<script lang="ts">
  import type { Snippet } from "svelte";
  import Icon from "./Icon.svelte";

  let {
    open = $bindable(false),
    title = "",
    logo = false,
    width = 560,
    onclose,
    children,
    footer,
  }: {
    open?: boolean;
    title?: string;
    /** Logo de l'app à gauche du titre (confirmations). */
    logo?: boolean;
    width?: number;
    onclose?: () => void;
    children: Snippet;
    footer?: Snippet;
  } = $props();

  function close() {
    open = false;
    onclose?.();
  }

  function onkey(e: KeyboardEvent) {
    if (e.key === "Escape" && open) {
      e.stopPropagation();
      close();
    }
  }
</script>

<svelte:window onkeydown={onkey} />

{#if open}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="backdrop" role="presentation" onclick={(e) => e.target === e.currentTarget && close()}>
    <div class="modal glass-strong fade-in" style="width:min({width}px, calc(100vw - 48px))" role="dialog" aria-modal="true" aria-label={title}>
      <header>
        <h2 class:with-logo={logo}>{#if logo}<img src="/logo.svg" alt="" width="28" height="28" />{/if}{title}</h2>
        <button class="btn ghost icon sm" onclick={close} aria-label="close"><Icon name="x" /></button>
      </header>
      <div class="body">{@render children()}</div>
      {#if footer}<footer>{@render footer()}</footer>{/if}
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: var(--titlebar-h) 0 0 0;
    background: rgba(0, 0, 0, 0.35);
    display: grid;
    place-items: center;
    z-index: 50;
    animation: fadeIn 0.18s ease;
  }
  .modal {
    /* Teinte du thème posée sur un fond plein : lisible même si le flou n'est pas rendu. */
    background:
      linear-gradient(var(--surface-2), var(--surface-2)),
      var(--bg-solid);
    max-height: calc(100vh - var(--titlebar-h) - 48px);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 18px 20px 6px;
  }
  .body {
    padding: 12px 20px 20px;
    overflow-y: auto;
  }
  footer {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    padding: 14px 20px;
    border-top: 1px solid var(--border);
  }
  h2.with-logo {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  h2.with-logo img {
    border-radius: 8px;
  }
</style>
