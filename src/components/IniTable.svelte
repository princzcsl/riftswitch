<script lang="ts">
  // Édition simple des sections game.cfg (interface, son, graphismes) d'un profil LoL.
  import type { IniDoc } from "../lib/types";
  import { t } from "../lib/i18n.svelte";

  let { doc = $bindable(), sections, onchange }: { doc: IniDoc; sections: string[]; onchange?: () => void } = $props();
  let q = $state("");

  const visible = $derived(
    doc.sections
      .map((s, si) => ({ s, si }))
      .filter(({ s }) => sections.includes(s.name))
      .map(({ s, si }) => ({
        name: s.name,
        si,
        rows: s.entries.map((e, ei) => ({ k: e[0], v: e[1], ei })).filter((r) => !q || r.k.toLowerCase().includes(q.toLowerCase())),
      }))
      .filter((s) => s.rows.length),
  );
</script>

<div class="col">
  <input class="input" placeholder={t("kb.searchSetting")} bind:value={q} />
  {#each visible as sec (sec.si)}
    <div class="section-title" style="margin:8px 0 0">{sec.name}</div>
    <div class="table">
      {#each sec.rows as r (r.ei)}
        <label class="tr">
          <span class="k mono small">{r.k}</span>
          {#if r.v === "0" || r.v === "1"}
            <input
              type="checkbox"
              checked={r.v === "1"}
              onchange={(e) => {
                doc.sections[sec.si].entries[r.ei][1] = e.currentTarget.checked ? "1" : "0";
                onchange?.();
              }}
            />
          {:else}
            <input
              class="input v"
              value={r.v}
              onchange={(e) => {
                doc.sections[sec.si].entries[r.ei][1] = e.currentTarget.value;
                onchange?.();
              }}
            />
          {/if}
        </label>
      {/each}
    </div>
  {:else}
    <p class="faint small">{t("kb.noSettings")}</p>
  {/each}
</div>

<style>
  .table {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
    gap: 2px 14px;
  }
  .tr {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 4px 8px;
    border-radius: 6px;
    min-height: 34px;
  }
  .tr:hover {
    background: var(--surface-hover);
  }
  .k {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    color: var(--text-2);
  }
  .v {
    width: 120px;
    height: 28px;
  }
</style>
