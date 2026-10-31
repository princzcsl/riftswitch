<script lang="ts">
  // Édition simple d'un INI de profil (game.cfg LoL, réglages VALORANT) : libellés lisibles, cases à cocher
  // pour les booléens, clés propres au PC masquées.
  import type { IniDoc } from "../lib/types";
  import { t } from "../lib/i18n.svelte";
  import { humanize, isVolatile } from "../lib/ini";

  let {
    doc = $bindable(),
    sections,
    filter,
    group,
    numericBools = true,
    onchange,
  }: {
    doc: IniDoc;
    /** Sections affichées (toutes si absent). */
    sections?: string[];
    filter?: (section: string, key: string) => boolean;
    /** Regroupement par clé (sinon par section). */
    group?: (key: string) => string;
    numericBools?: boolean;
    onchange?: () => void;
  } = $props();
  let q = $state("");

  const BOOLS: Record<string, string> = { "0": "1", "1": "0", true: "false", false: "true", True: "False", False: "True" };
  const isOn = (v: string) => v === "1" || v.toLowerCase() === "true";

  function label(kind: string): string {
    const key = `configs.sec_${kind}`;
    const v = t(key);
    return v === key ? humanize(kind) : v;
  }

  // Longs flottants (0.227500007) affichés arrondis ; la valeur n'est réécrite qu'en cas de modification.
  const shown = (v: string) => (/^-?\d+\.\d{4,}$/.test(v) ? String(Number(Number(v).toFixed(3))) : v);

  type Row = { k: string; v: string; si: number; ei: number };
  const groups = $derived.by(() => {
    const words = q.toLowerCase();
    const out = new Map<string, Row[]>();
    doc.sections.forEach((s, si) => {
      if (sections && !sections.includes(s.name)) return;
      s.entries.forEach(([k, v], ei) => {
        if (isVolatile(k) || (filter && !filter(s.name, k))) return;
        if (words && !humanize(k).toLowerCase().includes(words) && !k.toLowerCase().includes(words)) return;
        const g = group ? group(k) : s.name;
        if (!out.has(g)) out.set(g, []);
        out.get(g)!.push({ k, v, si, ei });
      });
    });
    return [...out.entries()];
  });

  function write(r: Row, value: string) {
    doc.sections[r.si].entries[r.ei][1] = value;
    onchange?.();
  }
</script>

<div class="col">
  <input class="input" placeholder={t("kb.searchSetting")} bind:value={q} />
  {#each groups as [name, rows] (name)}
    <div class="section-title" style="margin:8px 0 0">{label(name)}</div>
    <div class="table">
      {#each rows as r (`${r.si}:${r.ei}`)}
        <label class="tr">
          <span class="k">{humanize(r.k)}</span>
          {#if Object.prototype.hasOwnProperty.call(BOOLS, r.v) && (numericBools || !/^[01]$/.test(r.v))}
            <input type="checkbox" checked={isOn(r.v)} onchange={() => write(r, BOOLS[r.v])} />
          {:else}
            <input class="input v" value={shown(r.v)} onchange={(e) => write(r, e.currentTarget.value)} />
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
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
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
    white-space: nowrap;
    color: var(--text-2);
    font-size: 13px;
  }
  .v {
    width: 130px;
    height: 28px;
  }
</style>
