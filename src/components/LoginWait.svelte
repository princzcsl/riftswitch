<script lang="ts">
  // Ajout de compte via l'écran de connexion officiel du Riot Client (identifiants + A2F saisis chez Riot).
  import { api } from "../lib/api";
  import { app } from "../lib/app.svelte";
  import { errorText, t } from "../lib/i18n.svelte";
  import Icon from "./Icon.svelte";
  import Modal from "./Modal.svelte";

  let { open = $bindable(false), ondone }: { open?: boolean; ondone?: (id: string, created: boolean) => void } = $props();
  let running = $state(false);
  let error = $state("");

  $effect(() => {
    if (open && !running) start();
  });

  async function start() {
    running = true;
    error = "";
    app.loginHint = false;
    try {
      const r = await api.addViaRiotLogin();
      open = false;
      app.toast("success", r.created ? t("login.added") : t("login.updated"));
      await app.refreshAccounts();
      ondone?.(r.accountId, r.created);
    } catch (e) {
      const code = (e as { code?: string }).code;
      if (code !== "cancelled") error = errorText(e);
      else open = false;
    } finally {
      running = false;
    }
  }

  function cancel() {
    api.cancelLogin();
    open = false;
  }
</script>

<Modal bind:open title={t("login.title")} width={480} onclose={cancel}>
  <div class="wrap col">
    {#if error}
      <div class="err"><Icon name="alert" size={18} /><span>{error}</span></div>
      <button class="btn primary" onclick={start}>{t("common.retry")}</button>
    {:else}
      <div class="orb"><div class="ring"></div><img src="/logo.svg" alt="" width="54" height="54" /></div>
      <h3>{t("login.waiting")}</h3>
      <ol class="steps muted">
        <li>{t("login.step1")}</li>
        <li>{@html t("login.step2")}</li>
        <li>{t("login.step3")}</li>
      </ol>
      {#if app.loginHint}
        <div class="hint"><Icon name="info" size={16} /><span>{t("login.staySignedIn")}</span></div>
      {/if}
      <p class="faint small">{t("login.privacy")}</p>
    {/if}
  </div>
  {#snippet footer()}
    <button class="btn" onclick={cancel}>{t("common.cancel")}</button>
  {/snippet}
</Modal>

<style>
  .wrap {
    align-items: center;
    text-align: center;
    gap: 14px;
  }
  .orb {
    position: relative;
    width: 96px;
    height: 96px;
    display: grid;
    place-items: center;
  }
  .ring {
    position: absolute;
    inset: 0;
    border-radius: 50%;
    border: 3px solid transparent;
    border-top-color: var(--accent);
    border-right-color: var(--accent-2);
    animation: spin 1.2s linear infinite;
  }
  .steps {
    text-align: start;
    margin: 0;
    padding-left: 20px;
    line-height: 1.8;
  }
  .hint,
  .err {
    display: flex;
    gap: 8px;
    align-items: center;
    padding: 10px 12px;
    border-radius: var(--radius-sm);
    font-size: 13px;
    text-align: start;
  }
  .hint {
    background: color-mix(in srgb, var(--warning) 15%, transparent);
    color: var(--warning);
  }
  .err {
    background: color-mix(in srgb, var(--danger) 15%, transparent);
    color: var(--danger);
  }
</style>
