<script lang="ts">
  // Réglages d'un profil rangés comme dans le jeu (interrupteurs, curseurs, listes), le reste dans « Avancé ».
  import type { IniDoc } from "../lib/types";
  import { t } from "../lib/i18n.svelte";
  import { humanize, isVolatile } from "../lib/ini";
  import { bareKey, schemaKeys, type Field, type Group } from "../lib/settingsSchema";
  import IniTable from "./IniTable.svelte";
  import KeyCapture from "./KeyCapture.svelte";
  import Toggle from "./Toggle.svelte";

  let {
    doc = $bindable(),
    groups,
    advanced,
    numericBools = true,
    search = true,
    missing,
    onchange,
  }: {
    doc: IniDoc;
    groups: Group[];
    /** Clés de la catégorie affichée, pour la partie « Avancé ». */
    advanced: ((section: string, key: string) => boolean) | null;
    /** 0 / 1 lus comme des cases à cocher dans « Avancé » (LoL ; VALORANT écrit True / False). */
    numericBools?: boolean;
    search?: boolean;
    /** VALORANT : section où ajouter un réglage absent du fichier (le jeu n'écrit que les valeurs modifiées). */
    missing?: string;
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
  /** Champs numériques : chiffres et un seul point décimal, sans signe ni lettre. */
  function digitsOnly(e: Event & { currentTarget: HTMLInputElement }) {
    const el = e.currentTarget;
    const clean = el.value.replace(",", ".").replace(/[^0-9.]/g, "").replace(/(\.\d*)\./g, "$1");
    if (clean !== el.value) el.value = clean;
  }

  const CAN_ADD = ["bool", "select", "number"];
  /** Ajoute un réglage absent, avec le nom complet attendu par VALORANT. */
  function add(f: Field, v: string) {
    if (!missing || v === "") return;
    const type = f.vt ?? (f.kind === "bool" ? "Bool" : f.kind === "select" ? "Int" : "Float");
    let si = doc.sections.findIndex((s) => s.name === missing);
    if (si < 0) si = doc.sections.push({ name: missing, entries: [] }) - 1;
    doc.sections[si].entries.push([`EAres${type}SettingName::${f.key}`, v]);
    onchange?.();
  }

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
  /** Décimales affichées : celles du pas (0,001 → 3), aucune pour un pas entier. */
  const places = (f: Field) => (f.step && f.step < 1 ? String(f.step).split(".")[1].length : 0);
  const shown = (f: Field, n: number) => (f.pct ? `${Math.round(n * 100)} %` : String(Number(n.toFixed(places(f)))));
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
          .filter(({ f, pos }) => (f.kind === "mapSkins" ? mapSkins.length > 0 : !!pos || (!!missing && CAN_ADD.includes(f.kind))))
          .filter(({ f }) => !q || label(f).toLowerCase().includes(q.toLowerCase())),
      }))
      .filter((g) => g.fields.length),
  );

  const known = $derived(schemaKeys(groups));
  const extra = (s: string, k: string) => !!advanced && advanced(s, k) && !isVolatile(k) && !known.has(bareKey(k)) && !k.startsWith(MAP_SKIN);
  const extraCount = $derived(doc.sections.reduce((n, s) => n + s.entries.filter(([k]) => extra(s.name, k)).length, 0));
</script>

<div class="col">
  {#if search}<input class="input" placeholder={t("kb.searchSetting")} bind:value={q} />{/if}
  <div class="groups">
    {#each visible as g (g.id)}
      <div class="card">
        <div class="section-title">{t(`configs.g_${g.id}`)}</div>
        {#each g.fields as { f, pos } (f.key)}
          <div class="tr" class:wide={f.kind === "slider"}>
            {#if f.kind === "mapSkins"}
              <span class="k">{t("cfgk.MapSkins")}</span>
              <Toggle checked={mapSkins.every((e) => doc.sections[e.si].entries[e.ei][1] === "0")} onchange={setMapSkins} />
            {:else if pos}
              {@const v = val(pos)}
              <span class="k">{label(f)}</span>
              {#if f.kind === "bool"}
                <Toggle checked={isOn(v)} onchange={(on) => write(pos, boolText(v, on))} />
              {:else if f.kind === "slider"}
                {@const mute = f.mute ? find(f.mute) : null}
                {@const muted = !!mute && isOn(val(mute))}
                {#if mute}
                  <button class="mute" class:muted aria-label={t("cfgk.Mute")} onclick={() => write(mute, boolText(val(mute), !muted))}>
                    {t("cfgk.MuteShort")}
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
                  {#if !(f.options ?? []).includes(Math.round(Number(v)))}<option value={String(Math.round(Number(v)))}>{t("cfgopt.other", { v })}</option>{/if}
                </select>
              {:else if f.kind === "text"}
                <input class="input v" value={v} onchange={(e) => write(pos, e.currentTarget.value)} />
              {:else if f.kind === "key"}
                <KeyCapture mode="val" valKey={{ key: v, shift: false, ctrl: false, alt: false }} onval={(k) => write(pos, k?.key ?? "None")} />
              {:else}
                <input
                  class="input v"
                  inputmode="decimal"
                  oninput={digitsOnly}
                  value={shown(f, Number(v))}
                  onchange={(e) => write(pos, f.step && f.step < 1 ? e.currentTarget.value : numText(v, Number(e.currentTarget.value)))}
                />
              {/if}
            {:else}
              <span class="k">{label(f)}</span>
              {#if f.kind === "number"}
                <input class="input v" inputmode="decimal" oninput={digitsOnly} placeholder={t("cfgopt.default")} onchange={(e) => add(f, e.currentTarget.value)} />
              {:else}
                <select class="input sel default" value="" onchange={(e) => add(f, e.currentTarget.value)}>
                  <option value="">{t("cfgopt.default")}</option>
                  {#if f.kind === "bool"}
                    <option value="True">{t("cfgopt.on")}</option>
                    <option value="False">{t("cfgopt.off")}</option>
                  {:else}
                    {#each f.options ?? [] as o}<option value={String(o)}>{t(`cfgopt.${f.list}_${o}`)}</option>{/each}
                  {/if}
                </select>
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
      <IniTable bind:doc filter={extra} {numericBools} {onchange} />
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
  .tr.wide .k {
    flex: 0 0 34%;
  }
  .range {
    flex: 1;
    min-width: 100px;
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
    height: 24px;
    padding: 0 8px;
    border-radius: var(--radius-pill);
    border: 1px solid var(--border);
    background: none;
    color: var(--text-3);
    font-size: 11px;
    font-weight: 650;
    cursor: pointer;
  }
  .mute.muted {
    color: #fff;
    background: var(--danger);
    border-color: var(--danger);
  }
  .sel {
    width: 170px;
    height: 30px;
  }
  .sel.default {
    color: var(--text-3);
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
