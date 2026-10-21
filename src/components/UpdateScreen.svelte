<script lang="ts">
  // Écran plein, aux couleurs de l'app, pendant la mise à jour automatique au démarrage.
  import { t } from "../lib/i18n.svelte";
  import { updater } from "../lib/updater.svelte";

  const pct = $derived(updater.progress === null ? null : Math.round(updater.progress * 100));
</script>

{#if updater.updating}
  <div class="screen" role="status" aria-live="polite">
    <div class="glow"></div>
    <div class="box">
      <img src="/logo.svg" alt="" width="84" height="84" />
      <h1>{t("update.screenTitle")}</h1>
      {#if updater.available}<p class="muted">{t("update.screenText", { v: updater.available.version })}</p>{/if}
      <div class="bar" class:indeterminate={!pct}>
        <div style={pct ? `width:${pct}%` : ""}></div>
      </div>
      <span class="faint small">{pct ? `${pct} %` : t("update.preparing")}</span>
    </div>
  </div>
{:else if updater.startup}
  <div class="screen" role="status" aria-live="polite">
    <div class="glow"></div>
    <div class="box">
      <img src="/logo.svg" alt="" width="84" height="84" />
      <span class="faint small">{t("update.searching")}</span>
      <div class="bar indeterminate slim"><div></div></div>
    </div>
  </div>
{/if}

<style>
  .screen {
    position: fixed;
    inset: 0;
    z-index: 200;
    display: grid;
    place-items: center;
    background: var(--bg-solid);
    overflow: hidden;
  }
  .glow {
    position: absolute;
    inset: -30%;
    background:
      radial-gradient(closest-side at 35% 40%, color-mix(in srgb, var(--accent) 28%, transparent), transparent),
      radial-gradient(closest-side at 65% 65%, color-mix(in srgb, var(--accent-2) 18%, transparent), transparent);
    pointer-events: none;
  }
  .box {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    width: min(360px, calc(100vw - 32px));
    text-align: center;
  }
  h1 {
    font-size: 22px;
    margin-top: 6px;
  }
  .bar {
    width: 100%;
    height: 6px;
    border-radius: 3px;
    background: var(--surface-2);
    overflow: hidden;
    margin-top: 8px;
  }
  .bar div {
    height: 100%;
    border-radius: inherit;
    background: linear-gradient(90deg, var(--accent), var(--accent-2));
    transition: width 0.25s;
  }
  /* Taille inconnue : bande qui défile (informative, donc animée même en mode performance). */
  .bar.indeterminate div {
    width: 35%;
    animation: slide 1.2s ease-in-out infinite !important;
    animation-duration: 1.2s !important;
  }
  .bar.slim {
    width: 160px;
    height: 3px;
  }
  @keyframes slide {
    from {
      transform: translateX(-100%);
    }
    to {
      transform: translateX(290%);
    }
  }
</style>
