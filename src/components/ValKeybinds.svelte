<script lang="ts">
  // Éditeur de touches VALORANT (BackupKeybinds.json) : une ligne par action, liaison principale et secondaire.
  import { VAL_ACTIONS, VAL_GROUPS, actionLabel } from "../lib/actions";
  import type { ValKey } from "../lib/keys";
  import { i18n, t } from "../lib/i18n.svelte";
  import KeyCapture from "./KeyCapture.svelte";

  interface Mapping {
    name: string;
    characterName: string;
    bindIndex: number;
    key: string;
    shift: boolean;
    ctrl: boolean;
    alt: boolean;
    cmd: boolean;
    tapHoldType: string;
  }

  let { json = $bindable(""), onchange }: { json: string; onchange?: () => void } = $props();

  /** Liaison d'axe (déplacements) : une touche et un sens. */
  interface Axis {
    name: string;
    characterName: string;
    bindIndex: number;
    key: string;
    scale: number;
  }
  type Doc = { actionMappings?: Mapping[]; axisMappings?: Axis[]; [k: string]: unknown };

  let doc = $state<Doc | null>(null);
  let parseError = $state(false);
  let q = $state("");

  $effect(() => {
    try {
      doc = json ? JSON.parse(json) : { settingsVersion: 15, actionMappings: [], axisMappings: [] };
      parseError = false;
    } catch {
      parseError = true;
    }
  });

  function commit() {
    if (!doc) return;
    json = JSON.stringify(doc, null, "\t");
    onchange?.();
  }

  // Une ligne = une action (« Jump ») ou un axe et son sens (« axis:MoveRight:-1 »).
  const axisOf = (row: string) => {
    const m = row.match(/^axis:(\w+):(-?\d+)$/);
    return m ? { name: m[1], scale: Number(m[2]) } : null;
  };
  /** Toutes les liaisons d'une ligne, y compris les copies par agent que VALORANT enregistre. */
  function entries(row: string): (Mapping | Axis)[] {
    const ax = axisOf(row);
    if (ax) return (doc?.axisMappings ?? []).filter((m) => m.name === ax.name && Math.sign(m.scale) === Math.sign(ax.scale));
    return (doc?.actionMappings ?? []).filter((m) => m.name === row);
  }
  const slot = (row: string, i: number) => entries(row).find((m) => m.bindIndex === i && m.characterName === "None");
  const keyOf = (m: Mapping | Axis | undefined): ValKey | null =>
    m ? { key: m.key, shift: "shift" in m && m.shift, ctrl: "ctrl" in m && m.ctrl, alt: "alt" in m && m.alt } : null;
  const comboOf = (m: Mapping | Axis) => `${"ctrl" in m && m.ctrl}${"alt" in m && m.alt}${"shift" in m && m.shift}${m.key}`;

  const rows = $derived([
    ...new Set([
      ...VAL_ACTIONS.map((a) => a.id),
      ...(doc?.actionMappings ?? []).filter((m) => m.characterName === "None").map((m) => m.name),
    ]),
  ]);
  // Actions connues (même sans liaison) + celles présentes dans le fichier, par groupe.
  const groups = $derived.by(() => {
    const words = q.trim().toLowerCase();
    const by = new Map<string, string[]>();
    for (const n of rows) {
      if (words && !actionLabel(VAL_ACTIONS, n, i18n.lang).toLowerCase().includes(words) && !n.toLowerCase().includes(words)) continue;
      const g = VAL_ACTIONS.find((a) => a.id === n)?.group ?? "other";
      by.set(g, [...(by.get(g) ?? []), n]);
    }
    return [...VAL_GROUPS, "other"].filter((g) => by.has(g)).map((g) => ({ g, names: by.get(g)! }));
  });

  // Conflits : même touche sur deux lignes différentes (les copies par agent ne comptent pas).
  const conflicts = $derived.by(() => {
    const count = new Map<string, number>();
    for (const m of [...(doc?.actionMappings ?? []), ...(doc?.axisMappings ?? [])]) {
      if (m.characterName !== "None" || m.key === "None") continue;
      count.set(comboOf(m), (count.get(comboOf(m)) ?? 0) + 1);
    }
    return count;
  });

  /** Change une liaison partout où elle existe (y compris les copies par agent). */
  function setKey(row: string, i: number, v: ValKey | null) {
    if (!doc) return;
    const ax = axisOf(row);
    const list = (ax ? (doc.axisMappings ??= []) : (doc.actionMappings ??= [])) as (Mapping | Axis)[];
    const same = entries(row).filter((m) => m.bindIndex === i);
    if (!v) {
      for (const m of same) list.splice(list.indexOf(m), 1);
    } else if (same.length) {
      for (const m of same) Object.assign(m, ax ? { key: v.key } : { key: v.key, shift: v.shift, ctrl: v.ctrl, alt: v.alt });
    } else if (ax) {
      list.push({ name: ax.name, characterName: "None", bindIndex: i, key: v.key, scale: ax.scale });
    } else {
      list.push({ name: row, characterName: "None", bindIndex: i, key: v.key, shift: v.shift, ctrl: v.ctrl, alt: v.alt, cmd: false, tapHoldType: "None" });
    }
    commit();
  }
</script>

{#if parseError}
  <p class="small" style="color:var(--danger)">{t("kb.valParseError")}</p>
{:else}
  <div class="col">
    <div class="toolbar">
      <p class="faint small grow">{t("kb.valHint")}</p>
      <input class="input search" placeholder={t("kb.search")} bind:value={q} />
    </div>
    <div class="grid">
      {#each groups as { g, names } (g)}
        <div class="card">
          <div class="head">
            <span class="section-title grow">{t(`kb.vgroup_${g}`)}</span>
            <span class="slot-h">{t("kb.primary")}</span><span class="slot-h">{t("kb.secondary")}</span>
          </div>
          {#each names as name (name)}
            <div class="row-item">
              <span class="grow name">{actionLabel(VAL_ACTIONS, name, i18n.lang)}</span>
              {#each [0, 1] as i}
                {@const m = slot(name, i)}
                <KeyCapture
                  mode="val"
                  valKey={keyOf(m)}
                  conflict={!!m && (conflicts.get(comboOf(m)) ?? 0) > 1}
                  onval={(v) => setKey(name, i, v)}
                />
              {/each}
            </div>
          {/each}
        </div>
      {/each}
    </div>
  </div>
{/if}

<style>
  .toolbar {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .search {
    width: 200px;
    height: 32px;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(370px, 1fr));
    gap: 12px;
    align-items: start;
  }
  .card {
    padding: 10px 12px;
    border-radius: var(--radius-md);
    background: var(--surface);
    border: 1px solid var(--border);
  }
  .head {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 4px;
  }
  .head .section-title {
    margin: 0;
  }
  .slot-h {
    width: 108px;
    text-align: center;
    font-size: 10.5px;
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--text-3);
  }
  .row-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 0;
  }
  .name {
    font-weight: 540;
    min-width: 0;
  }
</style>
