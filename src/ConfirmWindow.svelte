<script lang="ts">
  // Mini-fenêtre de confirmation de fermeture, affichée seule (sans la fenêtre principale).
  import { onMount } from "svelte";
  import { api } from "./lib/api";
  import { setLanguage, t } from "./lib/i18n.svelte";
  import { applyTheme } from "./lib/themes";
  import Icon from "./components/Icon.svelte";

  let prompt = $state<{ ghost: boolean; inMatch: boolean } | null>(null);

  onMount(async () => {
    try {
      const boot = await api.bootstrap();
      setLanguage(boot.settings.language || "en");
      await applyTheme(boot.settings.theme, true);
    } catch {
      /* thème par défaut */
    }
    prompt = await api.exitPrompt().catch(() => null);
  });

  const answer = (ok: boolean) => api.answerExit(ok);
</script>

<svelte:window onkeydown={(e) => e.key === "Escape" && answer(false)} />

{#if prompt}
  <div class="card glass-solid fade-in" data-tauri-drag-region>
    <div class="head" data-tauri-drag-region>
      <span class="ico" class:danger={prompt.ghost}><Icon name={prompt.ghost ? "warning" : "info"} size={20} /></span>
      <strong data-tauri-drag-region>{t("exit.title")}</strong>
    </div>
    <p data-tauri-drag-region>
      {prompt.ghost ? t("exit.ghostText") + (prompt.inMatch ? " " + t("exit.abandon") : "") : t("exit.openText")}
    </p>
    <div class="actions">
      <button class="btn" onclick={() => answer(false)}>{t("common.cancel")}</button>
      <button class="btn {prompt.ghost ? 'danger solid' : 'primary'}" onclick={() => answer(true)}>
        {t(prompt.ghost ? "exit.quitClose" : "exit.quit")}
      </button>
    </div>
  </div>
{/if}

<style>
  :global(html.confirm-window),
  :global(html.confirm-window body) {
    background: transparent !important;
    overflow: hidden;
  }
  .card {
    position: fixed;
    inset: 6px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 18px 20px;
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.35);
  }
  .head {
    display: flex;
    align-items: center;
    gap: 12px;
    font-size: 15px;
  }
  .ico {
    display: grid;
    place-items: center;
    width: 36px;
    height: 36px;
    flex: none;
    border-radius: 50%;
    background: var(--accent-soft);
    color: var(--accent);
  }
  .ico.danger {
    background: color-mix(in srgb, var(--warning) 16%, transparent);
    color: var(--warning);
  }
  p {
    margin: 0;
    line-height: 1.5;
    color: var(--text-2);
  }
  .actions {
    margin-top: auto;
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }
</style>
