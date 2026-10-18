<script lang="ts">
  import type { Snippet } from "svelte";
  let { open = $bindable(false), x = 0, y = 0, children }: { open?: boolean; x?: number; y?: number; children: Snippet } = $props();
  let el = $state<HTMLDivElement>();
  // Repositionne pour ne pas déborder de la fenêtre.
  const left = $derived(Math.min(x, window.innerWidth - 220));
  const top = $derived(Math.min(y, window.innerHeight - 160));
  function onDoc(e: MouseEvent) {
    if (open && el && !el.contains(e.target as Node)) open = false;
  }
</script>

<svelte:window onmousedown={onDoc} onkeydown={(e) => e.key === "Escape" && (open = false)} onblur={() => (open = false)} />

{#if open}
  <div bind:this={el} class="ctx glass-strong fade-in" style="left:{left}px;top:{top}px" role="menu">
    {@render children()}
  </div>
{/if}

<style>
  .ctx {
    position: fixed;
    z-index: 80;
    min-width: 180px;
    padding: 6px;
    border-radius: var(--radius-md);
  }
  .ctx :global(.menu-item) {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    padding: 9px 10px;
    border: none;
    background: none;
    border-radius: var(--radius-sm);
    cursor: pointer;
    text-align: start;
    color: var(--text);
    font-weight: 520;
  }
  .ctx :global(.menu-item:hover) {
    background: var(--surface-hover);
  }
  .ctx :global(.menu-item.danger) {
    color: var(--danger);
  }
</style>
