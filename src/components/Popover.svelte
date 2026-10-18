<script lang="ts">
  // Menu flottant ancré à un déclencheur, fermé au clic extérieur ou à Échap.
  import type { Snippet } from "svelte";

  let {
    open = $bindable(false),
    align = "end",
    trigger,
    children,
    width = 220,
    up = false,
  }: {
    open?: boolean;
    align?: "start" | "end";
    trigger: Snippet;
    children: Snippet;
    width?: number;
    up?: boolean;
  } = $props();

  let root: HTMLDivElement;

  function onDoc(e: MouseEvent) {
    if (open && root && !root.contains(e.target as Node)) open = false;
  }
  function onKey(e: KeyboardEvent) {
    if (open && e.key === "Escape") open = false;
  }
</script>

<svelte:window onmousedown={onDoc} onkeydown={onKey} />

<div class="pop-root" bind:this={root}>
  {@render trigger()}
  {#if open}
    <div class="pop glass-strong fade-in" class:up style="{align === 'end' ? 'right:0' : 'left:0'}; width:{width}px">
      {@render children()}
    </div>
  {/if}
</div>

<style>
  .pop-root {
    position: relative;
    display: inline-flex;
  }
  .pop {
    position: absolute;
    top: calc(100% + 8px);
    z-index: 40;
    padding: 6px;
    border-radius: var(--radius-md);
  }
  .pop.up {
    top: auto;
    bottom: calc(100% + 8px);
  }
  .pop :global(.menu-item) {
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
  .pop :global(.menu-item:hover) {
    background: var(--surface-hover);
  }
  .pop :global(.menu-item.danger) {
    color: var(--danger);
  }
  .pop :global(.menu-item small) {
    display: block;
    color: var(--text-3);
    font-size: 11.5px;
    font-weight: 450;
  }
</style>
