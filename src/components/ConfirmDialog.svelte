<script lang="ts">
  // Confirmation au style de l'app (remplace les boîtes de dialogue Windows).
  import { app } from "../lib/app.svelte";
  import { t } from "../lib/i18n.svelte";
  import Icon from "./Icon.svelte";
  import Modal from "./Modal.svelte";

  // Ouverte tant qu'une confirmation attend ; Échap ou la croix valent « Annuler ».
  const getOpen = () => !!app.confirmState;
  const setOpen = (v: boolean) => {
    if (!v && app.confirmState) app.answerConfirm(false);
  };
</script>

{#if app.confirmState}
  {@const c = app.confirmState}
  <Modal bind:open={getOpen, setOpen} title={c.title} width={440}>
    <div class="row" style="gap:14px;align-items:flex-start">
      <span class="ico" class:danger={c.danger}><Icon name={c.danger ? "warning" : "info"} size={20} /></span>
      <p class="text">{c.text}</p>
    </div>
    {#snippet footer()}
      <button class="btn" onclick={() => app.answerConfirm(false)}>{c.cancel ?? t("common.cancel")}</button>
      <button class="btn {c.danger ? 'danger solid' : 'primary'}" onclick={() => app.answerConfirm(true)}>{c.ok}</button>
    {/snippet}
  </Modal>
{/if}

<style>
  .ico {
    display: grid;
    place-items: center;
    width: 38px;
    height: 38px;
    flex: none;
    border-radius: 50%;
    background: var(--accent-soft);
    color: var(--accent);
  }
  .ico.danger {
    background: color-mix(in srgb, var(--warning) 16%, transparent);
    color: var(--warning);
  }
  .text {
    margin: 6px 0 0;
    line-height: 1.5;
  }
</style>
