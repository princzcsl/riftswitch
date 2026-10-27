<script lang="ts">
  // Réglages d'un profil rangés comme dans le jeu (interrupteurs, curseurs, listes), le reste dans « Avancé ».
  import type { IniDoc } from "../lib/types";
  import { t } from "../lib/i18n.svelte";
  import { humanize, isVolatile } from "../lib/ini";
  import { bareKey, schemaKeys, type Field, type Group } from "../lib/settingsSchema";
  import Icon from "./Icon.svelte";
  import IniTable from "./IniTable.svelte";
  import Toggle from "./Toggle.svelte";

  let {
    doc = $bindable(),
    groups,
    advanced,
    onchange,
  }: {
    doc: IniDoc;
    groups: Group[];
    /** Clés de la catégorie affichée, pour la partie « Avancé ». */
    advanced: (section: string, key: string) => boolean;
    onchange?: () => void;
  } = $props();

  let q = $state("");
  const MAP_SKIN = "MapSkinOptionDisable";

  type Pos = { si: number; ei: number };
  function find(key: string): Pos | null {
    for (let si = 0; si < doc.sections.length; si++) {
      const ei = doc.sections[si].entries.findIndex(([k]) => bareKey(k) === key);
      if (ei >= 0) return { si, ei };
    }
    return null;
  }
  const val = (p: Pos) => doc.sections[p.si].entries[p.ei][1];
  function write(p: Pos, v: string) {
    doc.sections[p.si].entries[p.ei][1] = v;
    onchange?.();
  }

  const isOn = (v: string) => v === "1" || v.toLowerCase() === "true";
  /** Même écriture que la valeur d'origine : 0/1, true/false ou True/False. */
  function boolText(orig: string, on: boolean): string {
    if (orig === "0" || orig === "1") return on ? "1" : "0";
    if (orig === orig.toLowerCase()) return on ? "true" : "false";
    return on ? "True" : "False";
  }
  /** Garde le nombre de décimales de la valeur d'origine (6 au plus). */
  function numText(orig: string, n: number): string {
    const d = Math.min((orig.split(".")[1] ?? "").length, 6);
    return d ? n.toFixed(d) : String(Math.round(n));
  }
  const shown = (f: Field, n: number) => (f.pct ? `${Math.round(n * 100)} %` : String(Number(n.toFixed(f.step && f.step < 1 ? 2 : 0))));
  const label = (f: Field) => {
    const k = `cfgk.${f.label ?? f.key}`;
    const v = t(k);
    return v === k ? humanize(f.key) : v;
  };

  const mapSkins = $derived(
    doc.sections.flatMap((s, si) => s.entries.map(([k], ei) => ({ k, si, ei })).filter((e) => e.k.startsWith(MAP_SKIN))),
  );
  function setMapSkins(on: boolean) {
    for (const e of mapSkins) doc.sections[e.si].entries[e.ei][1] = on ? "0" : "1";
    onchange?.();
  }

  const visible = $derived(
    groups
      .map((g) => ({
        id: g.id,
        fields: g.fields
          .map((f) => ({ f, pos: f.kind === "mapSkins" ? null : find(f.key) }))
          .filter(({ f, pos }) => (f.kind === "mapSkins" ? mapSkins.length > 0 : !!pos))
          .filter(({ f }) => !q || label(f).toLowerCase().includes(q.toLowerCase())),
      }))
      .filter((g) => g.fields.length),
  );

  const known = $derived(schemaKeys(groups));
  const extra = (s: string, k: string) => advanced(s, k) && !isVolatile(k) && !known.has(bareKey(k)) && !k.startsWith(MAP_SKIN);
  const extraCount = $derived(doc.sections.reduce((n, s) => n + s.entries.filter(([k]) => extra(s.name, k)).length, 0));
</script>

<div class="col">
  <input class="input" placeholder={t("kb.searchSetting")} bind:value={q} />
  <div class="groups">
    {#each visible as g (g.id)}
      <div class="card">
        <div class="section-title">{t(`configs.g_${g.id}`)}</div>
        {#each g.fields as { f, pos } (f.key)}
          <div class="tr">
            {#if f.kind === "mapSkins"}
              <span class="k">{t("cfgk.MapSkins")}</span>
              <Toggle checked={mapSkins.every((e) => doc.sections[e.si].entries[e.ei][1] === "0")} onchange={setMapSkins} />
            {:else if pos}
              {@const v = val(pos)}
              <span class="k" title={f.key}>{label(f)}</span>
              {#if f.kind === "bool"}
                <Toggle checked={isOn(v)} onchange={(on) => write(pos, boolText(v, on))} />
              {:else if f.kind === "slider"}
                {@const mute = f.mute ? find(f.mute) : null}
                {@const muted = !!mute && isOn(val(mute))}
                {#if mute}
                  <button class="mute" class:muted title={t("cfgk.Mute")} onclick={() => write(mute, boolText(val(mute), !muted))}>
                    <Icon name={muted ? "x" : "signal"} size={13} />
                  </button>
                {/if}
                <input
                  class="range"
                  class:muted
                  type="range"
                  min={f.min}
                  max={f.max}
                  step={f.step}
                  value={Number(v)}
                  oninput={(e) => write(pos, numText(v, Number(e.currentTarget.value)))}
                />
                <span class="num">{shown(f, Number(v))}</span>
              {:else if f.kind === "select"}
                <select class="input sel" value={String(Math.round(Number(v)))} onchange={(e) => write(pos, numText(v, Number(e.currentTarget.value)))}>
                  {#each f.options ?? [] as o}<option value={String(o)}>{t(`cfgopt.${f.list}_${o}`)}</option>{/each}
                </select>
              {:else}
                <input class="input v" type="number" step={f.step ?? 1} value={v} onchange={(e) => write(pos, e.currentTarget.value)} />
              {/if}
            {/if}
          </div>
        {/each}
      </div>
    {/each}
  </div>
  {#if extraCount}
    <details class="adv">
      <summary>{t("configs.advanced")} <span class="faint">({extraCount})</span></summary>
      <IniTable bind:doc filter={extra} {onchange} />
    </details>
  {/if}
</div>

<style>
  .groups {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(340px, 1fr));
    gap: 12px;
    align-items: start;
  }
  .card {
    padding: 10px 12px;
    border-radius: var(--radius-md);
    background: var(--surface);
    border: 1px solid var(--border);
  }
  .card .section-title {
    margin: 2px 0 6px;
  }
  .tr {
    display: flex;
    align-items: center;
    gap: 10px;
    min-height: 36px;
    padding: 2px 0;
  }
  .k {
    flex: 1;
    min-width: 0;
    color: var(--text-2);
    font-size: 13px;
  }
  .range {
    width: 130px;
    accent-color: var(--accent);
  }
  .range.muted {
    opacity: 0.4;
  }
  .num {
    width: 44px;
    text-align: end;
    font-size: 12px;
    color: var(--text-3);
    font-variant-numeric: tabular-nums;
  }
  .mute {
    display: grid;
    place-items: center;
    width: 24px;
    height: 24px;
    border-radius: 6px;
    border: 1px solid var(--border);
    background: none;
    color: var(--text-3);
    cursor: pointer;
  }
  .mute.muted {
    color: var(--danger);
    border-color: var(--danger);
  }
  .sel {
    width: 170px;
    height: 30px;
  }
  .v {
    width: 110px;
    height: 30px;
  }
  .adv summary {
    cursor: pointer;
    color: var(--text-2);
    font-weight: 600;
    padding: 6px 0;
  }
</style>
