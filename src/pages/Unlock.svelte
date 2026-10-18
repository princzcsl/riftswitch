<script lang="ts">
  import { api } from "../lib/api";
  import { app } from "../lib/app.svelte";
  import { errorText, t } from "../lib/i18n.svelte";
  import Icon from "../components/Icon.svelte";
  import Modal from "../components/Modal.svelte";

  let pw = $state("");
  let busy = $state(false);
  let error = $state("");
  let resetOpen = $state(false);
  let resetStep = $state(0);
  let input = $state<HTMLInputElement>();

  const osMode = $derived(app.boot?.vaultMode === "os");

  $effect(() => {
    // Coffre sans mot de passe : déverrouillage automatique via la clé du système.
    if (osMode) submit();
    else input?.focus();
  });

  async function submit() {
    busy = true;
    error = "";
    try {
      await api.unlock(osMode ? null : pw);
      pw = "";
      app.onUnlocked();
    } catch (e) {
      error = errorText(e);
      pw = "";
    } finally {
      busy = false;
    }
  }

  async function reset() {
    if (resetStep === 0) return (resetStep = 1);
    await api.resetApp();
    location.reload();
  }
</script>

<div class="wrap">
  <div class="panel glass-strong fade-in">
    <img src="/logo.svg" alt="" width="84" height="84" />
    <h1>RiftSwitch</h1>
    {#if osMode}
      <p class="muted">{busy ? t("unlock.opening") : error || t("unlock.opening")}</p>
      {#if error}<button class="btn primary" onclick={submit}>{t("common.retry")}</button>{/if}
    {:else}
      <p class="muted">{t("unlock.prompt")}</p>
      <form
        class="row"
        onsubmit={(e) => {
          e.preventDefault();
          submit();
        }}
      >
        <div class="pw">
          <Icon name="lock" size={16} />
          <input bind:this={input} type="password" autocomplete="current-password" placeholder={t("unlock.placeholder")} bind:value={pw} disabled={busy} />
        </div>
        <button class="btn primary" type="submit" disabled={busy || !pw}>
          {#if busy}<svg class="spin" width="14" height="14" viewBox="0 0 24 24"><path d="M12 3a9 9 0 1 0 9 9" stroke="currentColor" stroke-width="3" fill="none" /></svg>{:else}<Icon name="chevronRight" size={16} />{/if}
        </button>
      </form>
      {#if error}<p class="error">{error}</p>{/if}
    {/if}
    <button class="link small" onclick={() => ((resetOpen = true), (resetStep = 0))}>{t("unlock.forgot")}</button>
  </div>
</div>

<Modal bind:open={resetOpen} title={t("settings.resetTitle")} width={440}>
  <p>{t("unlock.forgotText")}</p>
  {#if resetStep === 1}<p class="error">{t("settings.resetConfirm2")}</p>{/if}
  {#snippet footer()}
    <button class="btn" onclick={() => (resetOpen = false)}>{t("common.cancel")}</button>
    <button class="btn danger solid" onclick={reset}>{resetStep === 0 ? t("settings.reset") : t("settings.resetFinal")}</button>
  {/snippet}
</Modal>

<style>
  .wrap {
    height: 100%;
    display: grid;
    place-items: center;
  }
  .panel {
    width: 380px;
    padding: 34px 30px 22px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
    text-align: center;
  }
  .panel img {
    filter: drop-shadow(0 12px 30px var(--accent-soft));
  }
  form {
    width: 100%;
    margin-top: 8px;
  }
  .pw {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 8px;
    height: 42px;
    padding: 0 12px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--text-3);
  }
  .pw:focus-within {
    border-color: var(--accent);
    box-shadow: 0 0 0 3px var(--accent-soft);
  }
  .pw input {
    flex: 1;
    border: none;
    outline: none;
    background: transparent;
    color: var(--text);
    user-select: text;
  }
  form .btn {
    height: 42px;
    width: 46px;
    padding: 0;
  }
  .error {
    color: var(--danger);
    margin: 0;
    font-size: 13px;
  }
  .link {
    margin-top: 10px;
    border: none;
    background: none;
    color: var(--text-3);
    cursor: pointer;
    text-decoration: underline;
  }
</style>
