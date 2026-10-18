<script lang="ts">
  import { app } from "../lib/app.svelte";
  import { errorText, t } from "../lib/i18n.svelte";
  import { updater } from "../lib/updater.svelte";
  import Icon from "./Icon.svelte";

  async function install() {
    try {
      await updater.install();
    } catch (e) {
      app.toast("error", `${t("update.failed")} ${errorText(e)}`);
    }
  }
</script>

{#if updater.available && !updater.dismissed}
  <div class="banner glass-strong fade-in" role="status">
    <span class="ico"><Icon name="sparkle" size={18} /></span>
    <div class="grow col" style="gap:2px">
      <strong>{t("update.available", { v: updater.available.version })}</strong>
      {#if updater.progress === null}
        <span class="small muted">{t("update.text")}</span>
      {:else}
        <span class="small muted">{t("update.downloading", { p: Math.round(updater.progress * 100) })}</span>
        <div class="bar"><div style="width:{Math.round(updater.progress * 100)}%"></div></div>
      {/if}
    </div>
    {#if updater.progress === null}
      <button class="btn sm ghost" onclick={() => (updater.dismissed = true)}>{t("update.later")}</button>
      <button class="btn sm primary" onclick={install}><Icon name="download" size={14} />{t("update.install")}</button>
    {/if}
  </div>
{/if}

<style>
  .banner {
    position: fixed;
    right: 20px;
    bottom: 20px;
    z-index: 70;
    width: min(420px, calc(100vw - 32px));
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 14px 16px;
    border-radius: var(--radius-md);
  }
  .ico {
    display: grid;
    place-items: center;
    width: 34px;
    height: 34px;
    border-radius: 50%;
    background: var(--accent-soft);
    color: var(--accent);
    flex: none;
  }
  .bar {
    height: 4px;
    border-radius: 2px;
    background: var(--surface-2);
    overflow: hidden;
    margin-top: 4px;
  }
  .bar div {
    height: 100%;
    background: linear-gradient(90deg, var(--accent), var(--accent-2));
    transition: width 0.2s;
  }
</style>
