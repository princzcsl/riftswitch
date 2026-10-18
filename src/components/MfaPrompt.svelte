<script lang="ts">
  // Demande du code A2F pendant une connexion assistée. Le code est saisi par l'utilisateur.
  import { api } from "../lib/api";
  import { app } from "../lib/app.svelte";
  import { errorText, t } from "../lib/i18n.svelte";
  import Icon from "./Icon.svelte";
  import Modal from "./Modal.svelte";

  let code = $state("");
  let busy = $state(false);
  const open = $derived(app.mfa !== null);

  async function submit() {
    if (!code.trim()) return;
    busy = true;
    try {
      await api.submitMfa(code.trim());
      app.mfa = null;
      code = "";
    } catch (e) {
      app.toast("error", errorText(e));
    } finally {
      busy = false;
    }
  }

  function cancel() {
    api.cancelLogin();
    app.mfa = null;
    code = "";
  }
</script>

<Modal {open} title={t("mfa.title")} width={420} onclose={cancel}>
  <div class="col" style="gap:14px">
    <div class="row" style="gap:10px"><Icon name="shield" size={20} /><p class="muted" style="margin:0">{t("mfa.text", { email: app.mfa?.email ?? "" })}</p></div>
    <input
      class="input code"
      inputmode="numeric"
      autocomplete="one-time-code"
      maxlength="8"
      placeholder="000000"
      bind:value={code}
      onkeydown={(e) => e.key === "Enter" && submit()}
    />
  </div>
  {#snippet footer()}
    <button class="btn" onclick={cancel}>{t("common.cancel")}</button>
    <button class="btn primary" onclick={submit} disabled={busy || !code.trim()}>{t("mfa.confirm")}</button>
  {/snippet}
</Modal>

<style>
  .code {
    font-size: 24px;
    letter-spacing: 8px;
    text-align: center;
    font-family: "Cascadia Code", monospace;
    height: 52px;
  }
</style>
