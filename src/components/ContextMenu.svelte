<script lang="ts">
  import type { Snippet } from "svelte";
  let {
    open = $bindable(false),
    x = 0,
    y = 0,
    fromBottom = false,
    children,
  }: { open?: boolean; x?: number; y?: number; fromBottom?: boolean; children: Snippet } = $props();
  let el = $state<HTMLDivElement>();
  let pos = $state<{ left: number; top: number } | null>(null);

  // Placé après mesure : `fromBottom` aligne le bas du menu sur `y`, et il reste dans la fenêtre.
  $effect(() => {
    if (!open || !el) {
      pos = null;
      return;
    }
    const { width, height } = el.getBoundingClientRect();
    const top = fromBottom ? y - height : y;
    pos = {
      left: Math.max(8, Math.min(x, window.innerWidth - width - 8)),
      top: Math.max(8, Math.min(top, window.innerHeight - height - 8)),
    };
  });

  function onDoc(e: MouseEvent) {
    if (open && el && !el.contains(e.target as Node)) open = false;
  }
</script>

<svelte:window onmousedown={onDoc} onkeydown={(e) => e.key === "Escape" && (open = false)} onblur={() => (open = false)} />

{#if open}
  <div
    bind:this={el}
    class="ctx glass-strong fade-in"
    style="left:{pos?.left ?? 0}px;top:{pos?.top ?? 0}px;visibility:{pos ? 'visible' : 'hidden'}"
    role="menu"
  >
    {@render children()}
  </div>
{/if}

<style>
  .ctx {
    position: fixed;
    z-index: 80;
    background:
      linear-gradient(var(--surface-2), var(--surface-2)),
      var(--bg-solid);
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
