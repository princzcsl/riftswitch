<script lang="ts">
  // Nouveautés : affichées une fois après une mise à jour, et depuis Paramètres › À propos.
  import { app } from "../lib/app.svelte";
  import { releasesSince } from "../lib/changelog";
  import { i18n, t } from "../lib/i18n.svelte";
  import Modal from "./Modal.svelte";

  const releases = $derived(app.whatsNew === null ? [] : releasesSince(app.whatsNew, app.boot?.version ?? ""));
  const lang = $derived(i18n.lang.split("-")[0]);
</script>

<Modal bind:open={() => app.whatsNew !== null && releases.length > 0, (v) => !v && (app.whatsNew = null)} title={t("whatsNew.title")} width={560} logo>
  <div class="col notes">
    {#each releases as r (r.version)}
      {@const n = r.notes[lang] ?? r.notes.en}
      <div class="section-title">{t("whatsNew.version", { v: r.version })}</div>
      {#each [["added", n.added], ["changed", n.changed], ["fixed", n.fixed]] as [kind, items] (kind)}
        {#if items?.length}
          <strong class="kind {kind}">{t(`whatsNew.${kind}`)}</strong>
          <ul>
            {#each items as line}<li>{line}</li>{/each}
          </ul>
        {/if}
      {/each}
    {/each}
  </div>
  {#snippet footer()}
    <button class="btn primary" onclick={() => (app.whatsNew = null)}>{t("whatsNew.ok")}</button>
  {/snippet}
</Modal>

<style>
  .notes {
    max-height: 60vh;
    overflow-y: auto;
    gap: 6px;
  }
  .kind {
    font-size: 13px;
    margin-top: 4px;
  }
  .kind.added {
    color: var(--success);
  }
  .kind.changed {
    color: var(--info);
  }
  .kind.fixed {
    color: var(--warning);
  }
  ul {
    margin: 0;
    padding-left: 18px;
    color: var(--text-2);
    font-size: 13px;
    line-height: 1.5;
  }
</style>
