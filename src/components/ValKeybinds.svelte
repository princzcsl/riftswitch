<script lang="ts">
  // Éditeur de touches VALORANT (BackupKeybinds.json) : liaisons personnalisées par action.
  import { VAL_ACTIONS, VAL_GROUPS, actionLabel } from "../lib/actions";
  import type { ValKey } from "../lib/keys";
  import { i18n, t } from "../lib/i18n.svelte";
  import Icon from "./Icon.svelte";
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
  let adding = $state("");

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
  const grouped = $derived.by(() => {
    const byGroup = new Map<string, number[]>();
    mappings.forEach((m, i) => {
      const g = VAL_ACTIONS.find((a) => a.id === m.name)?.group ?? "other";
      byGroup.set(g, [...(byGroup.get(g) ?? []), i]);
    });
    return [...VAL_GROUPS, "other"].filter((g) => byGroup.has(g)).map((g) => ({ g, idx: byGroup.get(g)! }));
  });

  const conflicts = $derived.by(() => {
    const count = new Map<string, number>();
    for (const m of mappings) {
      const k = `${m.ctrl}${m.alt}${m.shift}${m.key}`;
      count.set(k, (count.get(k) ?? 0) + 1);
    }
    return count;
  });

  function setKey(i: number, v: ValKey | null) {
    if (!doc?.actionMappings) return;
    const m = doc.actionMappings[i];
    if (!v) {
      doc.actionMappings.splice(i, 1);
    } else {
      m.key = v.key;
      m.shift = v.shift;
      m.ctrl = v.ctrl;
      m.alt = v.alt;
    }
    commit();
  }

  function add() {
    if (!adding || !doc) return;
    doc.actionMappings ??= [];
    const used = doc.actionMappings.filter((m) => m.name === adding).map((m) => m.bindIndex);
    doc.actionMappings.push({
      name: adding,
      characterName: "None",
      bindIndex: used.includes(0) ? 1 : 0,
      key: "None",
      shift: false,
      ctrl: false,
      alt: false,
      cmd: false,
      tapHoldType: "None",
    });
    adding = "";
    commit();
  }
</script>

{#if parseError}
  <p class="small" style="color:var(--danger)">{t("kb.valParseError")}</p>
{:else}
  <div class="col">
    <p class="faint small">{t("kb.valHint")}</p>
    {#each grouped as { g, idx }}
      <div class="section-title" style="margin:8px 0 2px">{t(`kb.vgroup_${g}`)}</div>
      {#each idx as i (i)}
        {@const m = mappings[i]}
        <div class="row-item">
          <div class="grow name">
            <span>{actionLabel(VAL_ACTIONS, m.name, i18n.lang)}</span>
            <span class="faint small">{m.bindIndex === 0 ? t("kb.primary") : t("kb.secondary")}{m.characterName !== "None" ? ` · ${m.characterName}` : ""}</span>
          </div>
          <KeyCapture
            mode="val"
            valKey={{ key: m.key, shift: m.shift, ctrl: m.ctrl, alt: m.alt }}
            conflict={(conflicts.get(`${m.ctrl}${m.alt}${m.shift}${m.key}`) ?? 0) > 1 && m.key !== "None"}
            onval={(v) => setKey(i, v)}
          />
          <button class="btn ghost icon sm" onclick={() => setKey(i, null)} title={t("common.delete")}><Icon name="trash" size={15} /></button>
        </div>
      {/each}
    {/each}
    <div class="row add">
      <select class="input grow" bind:value={adding}>
        <option value="">{t("kb.addBinding")}</option>
        {#each VAL_ACTIONS as a}<option value={a.id}>{actionLabel(VAL_ACTIONS, a.id, i18n.lang)}</option>{/each}
      </select>
      <button class="btn" onclick={add} disabled={!adding}><Icon name="plus" size={15} />{t("common.add")}</button>
    </div>
  </div>
{/if}

<style>
  .row-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 10px;
    border-radius: var(--radius-sm);
  }
  .row-item:hover {
    background: var(--surface-hover);
  }
  .name {
    display: flex;
    flex-direction: column;
    font-weight: 560;
  }
  .add {
    margin-top: 10px;
  }
</style>
