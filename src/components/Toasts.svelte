<script lang="ts">
  import { app } from "../lib/app.svelte";
  import Icon from "./Icon.svelte";
  const icons = { info: "info", success: "check", error: "alert", warning: "alert" } as const;
</script>

<div class="toasts" aria-live="polite">
  {#each app.toasts as toast (toast.id)}
    <div class="toast glass-strong fade-in {toast.kind}">
      <Icon name={icons[toast.kind]} size={16} />
      <span>{toast.text}</span>
    </div>
  {/each}
</div>

<style>
  .toasts {
    position: fixed;
    right: 20px;
    bottom: 20px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    z-index: 100;
    pointer-events: none;
  }
  .toast {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 11px 15px;
    border-radius: var(--radius-md);
    max-width: 420px;
    font-weight: 550;
    pointer-events: auto;
  }
  .success :global(svg) {
    color: var(--success);
  }
  .error :global(svg) {
    color: var(--danger);
  }
  .warning :global(svg) {
    color: var(--warning);
  }
  .info :global(svg) {
    color: var(--info);
  }
</style>
