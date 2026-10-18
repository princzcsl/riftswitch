<script lang="ts">
  // Éditeur de touches League of Legends (input.ini) : 2 liaisons par action, conflits signalés.
  import type { IniDoc } from "../lib/types";
  import { LOL_ACTIONS, LOL_GROUPS, actionLabel, humanize } from "../lib/actions";
  import { LOL_UNBOUND, joinLolValue, splitLolValue } from "../lib/keys";
  import { i18n, t } from "../lib/i18n.svelte";
  import KeyCapture from "./KeyCapture.svelte";

  let { input = $bindable(), onchange }: { input: IniDoc; onchange?: () => void } = $props();

  let group = $state<string>("spells");
  let q = $state("");

  function find(id: string): { s: number; e: number } | null {
    for (let s = 0; s < input.sections.length; s++) {
      const e = input.sections[s].entries.findIndex(([k]) => k === id);
      if (e >= 0) return { s, e };
    }
    return null;
  }

  function get(id: string): string[] {
    const pos = find(id);
    if (!pos) return [];
    return splitLolValue(input.sections[pos.s].entries[pos.e][1]);
  }

  function set(id: string, slot: number, binding: string) {
    const current = get(id);
    while (current.length < 2) current.push(LOL_UNBOUND);
    current[slot] = binding;
    const value = joinLolValue(current);
    const pos = find(id);
    if (pos) input.sections[pos.s].entries[pos.e][1] = value;
    else {
      let s = input.sections.findIndex((x) => x.name === "GameEvents");
      if (s < 0) {
        input.sections.push({ name: "GameEvents", entries: [] });
        s = input.sections.length - 1;
      }
      input.sections[s].entries.push([id, value]);
    }
    onchange?.();
  }

  // Liaisons utilisées plusieurs fois (hors sections Quickbinds) → signalées en orange.
  const conflicts = $derived.by(() => {
    const count = new Map<string, number>();
    for (const sec of input.sections) {
      if (sec.name === "Quickbinds") continue;
      for (const [, v] of sec.entries) for (const b of splitLolValue(v)) if (b && b !== LOL_UNBOUND) count.set(b, (count.get(b) ?? 0) + 1);
    }
    return new Set([...count].filter(([, n]) => n > 1).map(([b]) => b));
  });

  const rows = $derived.by(() => {
    const s = q.trim().toLowerCase();
    if (group === "all" || s) {
      const all: { id: string; section: string }[] = [];
      for (const sec of input.sections) {
        if (sec.name === "Quickbinds") continue;
        for (const [k] of sec.entries) all.push({ id: k, section: sec.name });
      }
      return all
        .filter((r) => !s || r.id.toLowerCase().includes(s) || actionLabel(LOL_ACTIONS, r.id, i18n.lang).toLowerCase().includes(s))
        .map((r) => r.id);
    }
    return LOL_ACTIONS.filter((a) => a.group === group).map((a) => a.id);
  });

  const quick = $derived(input.sections.find((x) => x.name === "Quickbinds"));
</script>

<div class="kb">
  <div class="toolbar">
    <div class="groups">
      {#each LOL_GROUPS as g}
        <button class:on={group === g && !q} onclick={() => ((group = g), (q = ""))}>{t(`kb.group_${g}`)}</button>
      {/each}
      <button class:on={group === "all" && !q} onclick={() => ((group = "all"), (q = ""))}>{t("kb.group_all")}</button>
      {#if quick}<button class:on={group === "quick" && !q} onclick={() => ((group = "quick"), (q = ""))}>{t("kb.group_quick")}</button>{/if}
    </div>
    <input class="input search" placeholder={t("kb.search")} bind:value={q} />
  </div>

  {#if group === "quick" && quick && !q}
    <p class="faint small">{t("kb.quickHint")}</p>
    <div class="list">
      {#each quick.entries as entry, i}
        <label class="row-item">
          <span class="grow">{actionLabel(LOL_ACTIONS, entry[0].replace(/smart$/, ""), i18n.lang)}</span>
          <span class="faint small mono">{entry[0]}</span>
          <input
            type="checkbox"
            checked={entry[1] === "1"}
            onchange={(e) => {
              quick.entries[i][1] = e.currentTarget.checked ? "1" : "0";
              onchange?.();
            }}
          />
        </label>
      {/each}
    </div>
  {:else}
    <div class="list">
      <div class="head small faint">
        <span class="grow">{t("kb.action")}</span><span class="slot-h">{t("kb.primary")}</span><span class="slot-h">{t("kb.secondary")}</span>
      </div>
      {#each rows as id (id)}
        {@const binds = get(id)}
        <div class="row-item">
          <div class="grow name">
            <span>{actionLabel(LOL_ACTIONS, id, i18n.lang)}</span>
            {#if !LOL_ACTIONS.some((a) => a.id === id)}<span class="faint small mono">{id}</span>{:else if group === "all" || q}<span class="faint small">{humanize(id)}</span>{/if}
          </div>
          {#each [0, 1] as slot}
            <KeyCapture
              mode="lol"
              value={binds[slot] ?? ""}
              conflict={!!binds[slot] && conflicts.has(binds[slot])}
              onlol={(v) => set(id, slot, v)}
            />
          {/each}
        </div>
      {:else}
        <p class="faint small" style="padding:12px">{t("kb.noResult")}</p>
      {/each}
    </div>
  {/if}
</div>

<style>
  .kb {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .toolbar {
    display: flex;
    gap: 10px;
    align-items: center;
    flex-wrap: wrap;
  }
  .groups {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    flex: 1;
  }
  .groups button {
    border: 1px solid var(--border);
    background: transparent;
    color: var(--text-2);
    padding: 6px 11px;
    border-radius: var(--radius-pill);
    cursor: pointer;
    font-weight: 600;
    font-size: 12px;
  }
  .groups button.on {
    background: var(--accent-soft);
    color: var(--text);
    border-color: transparent;
  }
  .search {
    width: 200px;
    height: 32px;
  }
  .list {
    display: flex;
    flex-direction: column;
  }
  .head {
    display: flex;
    gap: 8px;
    padding: 0 10px 6px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    font-weight: 700;
    font-size: 10.5px;
  }
  .slot-h {
    width: 108px;
    text-align: center;
  }
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
</style>
