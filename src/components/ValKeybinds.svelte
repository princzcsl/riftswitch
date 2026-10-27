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

  let doc = $state<{ actionMappings?: Mapping[]; [k: string]: unknown } | null>(null);
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

  const mappings = $derived((doc?.actionMappings ?? []) as Mapping[]);
  // Actions connues (même sans liaison) + celles présentes dans le fichier, par groupe.
  const groups = $derived.by(() => {
    const words = q.trim().toLowerCase();
    const names = [...new Set([...VAL_ACTIONS.map((a) => a.id), ...mappings.filter((m) => m.characterName === "None").map((m) => m.name)])];
    const by = new Map<string, string[]>();
    for (const n of names) {
      if (words && !actionLabel(VAL_ACTIONS, n, i18n.lang).toLowerCase().includes(words) && !n.toLowerCase().includes(words)) continue;
      const g = VAL_ACTIONS.find((a) => a.id === n)?.group ?? "other";
      by.set(g, [...(by.get(g) ?? []), n]);
    }
    return [...VAL_GROUPS, "other"].filter((g) => by.has(g)).map((g) => ({ g, names: by.get(g)! }));
  });

  const conflicts = $derived.by(() => {
    const count = new Map<string, number>();
    for (const m of mappings) {
      const k = `${m.ctrl}${m.alt}${m.shift}${m.key}`;
      count.set(k, (count.get(k) ?? 0) + 1);
    }
    return count;
  });

  const slot = (name: string, i: number) => mappings.find((m) => m.name === name && m.bindIndex === i && m.characterName === "None");
  const keyOf = (m: Mapping | undefined): ValKey | null => (m ? { key: m.key, shift: m.shift, ctrl: m.ctrl, alt: m.alt } : null);

  function setKey(name: string, i: number, v: ValKey | null) {
    if (!doc) return;
    doc.actionMappings ??= [];
    const list = doc.actionMappings;
    const at = list.findIndex((m) => m.name === name && m.bindIndex === i && m.characterName === "None");
    if (!v) {
      if (at >= 0) list.splice(at, 1);
    } else if (at >= 0) {
      Object.assign(list[at], { key: v.key, shift: v.shift, ctrl: v.ctrl, alt: v.alt });
    } else {
      list.push({ name, characterName: "None", bindIndex: i, key: v.key, shift: v.shift, ctrl: v.ctrl, alt: v.alt, cmd: false, tapHoldType: "None" });
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
                  conflict={!!m && m.key !== "None" && (conflicts.get(`${m.ctrl}${m.alt}${m.shift}${m.key}`) ?? 0) > 1}
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
