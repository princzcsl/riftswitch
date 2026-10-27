<script lang="ts">
  // Confirmation au style de l'app (remplace les boîtes de dialogue Windows).
  import { app } from "../lib/app.svelte";
  import { t } from "../lib/i18n.svelte";
  import Modal from "./Modal.svelte";

  // Ouverte tant qu'une confirmation attend ; Échap ou la croix valent « Annuler ».
  const getOpen = () => !!app.confirmState;
  const setOpen = (v: boolean) => {
    if (!v && app.confirmState) app.answerConfirm(false);
  };
</script>

{#if app.confirmState}
  {@const c = app.confirmState}
  <Modal bind:open={getOpen, setOpen} title={c.title} logo width={440}>
    <p class="text">{c.text}</p>
    {#snippet footer()}
      <button class="btn" onclick={() => app.answerConfirm(false)}>{c.cancel ?? t("common.cancel")}</button>
      <button class="btn {c.danger ? 'danger solid' : 'primary'}" onclick={() => app.answerConfirm(true)}>{c.ok}</button>
    {/snippet}
  </Modal>
{/if}

<style>
  .text {
    margin: 0;
    line-height: 1.5;
  }
</style>
