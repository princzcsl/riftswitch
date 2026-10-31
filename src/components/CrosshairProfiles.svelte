<script lang="ts">
  // Profils de viseur VALORANT (SavedCrosshairProfileData) : aperçu, viseur actif, retrait du profil et
  // copie du code de viseur à coller dans le jeu.
  import type { IniDoc } from "../lib/types";
  import { app } from "../lib/app.svelte";
  import { t } from "../lib/i18n.svelte";
  import { bareKey } from "../lib/settingsSchema";
  import Icon from "./Icon.svelte";

  type Rgba = { r: number; g: number; b: number; a: number };
  type Lines = {
    lineThickness: number;
    lineLength: number;
    lineLengthVertical: number;
    lineOffset: number;
    bAllowVertScaling: boolean;
    opacity: number;
    bShowLines: boolean;
    bShowMovementError: boolean;
    bShowShootingError: boolean;
  };
  type Primary = {
    color: Rgba;
    colorCustom: Rgba;
    bUseCustomColor: boolean;
    bHasOutline: boolean;
    outlineThickness: number;
    outlineColor: Rgba;
    outlineOpacity: number;
    centerDotSize: number;
    centerDotOpacity: number;
    bDisplayCenterDot: boolean;
    innerLines: Lines;
    outerLines: Lines;
  };
  type Profile = { profileName: string; primary: Primary };
  type Data = { currentProfile: number; profiles: Profile[] };

  let { doc = $bindable(), onchange }: { doc: IniDoc; onchange?: () => void } = $props();

  function pos(): { si: number; ei: number } | null {
    for (let si = 0; si < doc.sections.length; si++) {
      const ei = doc.sections[si].entries.findIndex(([k]) => bareKey(k) === "SavedCrosshairProfileData");
      if (ei >= 0) return { si, ei };
    }
    return null;
  }

  // La valeur INI est une chaîne JSON qui contient elle-même du JSON.
  const data = $derived.by<Data | null>(() => {
    const p = pos();
    if (!p) return null;
    try {
      return JSON.parse(JSON.parse(doc.sections[p.si].entries[p.ei][1]));
    } catch {
      return null;
    }
  });

  function save(next: Data) {
    const p = pos();
    if (!p) return;
    doc.sections[p.si].entries[p.ei][1] = JSON.stringify(JSON.stringify(next));
    onchange?.();
  }

  const choose = (i: number) => data && save({ ...data, currentProfile: i });

  /** Retire un viseur de ce profil (il ne sera pas ajouté au compte cible). */
  function remove(i: number) {
    if (!data || data.profiles.length < 2) return;
    const profiles = data.profiles.filter((_, j) => j !== i);
    const cur = data.currentProfile;
    save({ ...data, profiles, currentProfile: cur === i ? 0 : cur > i ? cur - 1 : cur });
  }

  // Code de viseur VALORANT (Paramètres › Viseur › Importer) : viseur principal.
  const PRESETS = [
    [255, 255, 255],
    [0, 255, 0],
    [127, 255, 0],
    [223, 255, 0],
    [255, 255, 0],
    [0, 255, 255],
    [255, 0, 255],
    [255, 0, 0],
  ];
  const num = (x: number) => String(Number((x ?? 0).toFixed(3)));
  const bit = (x: boolean) => (x ? "1" : "0");
  function colorPart(p: Primary): string {
    const c = p.bUseCustomColor ? p.colorCustom : p.color;
    const i = p.bUseCustomColor ? -1 : PRESETS.findIndex(([r, g, b]) => r === c.r && g === c.g && b === c.b);
    if (i >= 0) return `c;${i}`;
    const hex = [c.r, c.g, c.b, c.a ?? 255].map((n) => n.toString(16).padStart(2, "0")).join("").toUpperCase();
    return `c;8;u;${hex}`;
  }
  const lines = (k: string, l: Lines) =>
    [
      `${k}b;${bit(l.bShowLines)}`,
      `${k}t;${num(l.lineThickness)}`,
      `${k}l;${num(l.lineLength)}`,
      `${k}v;${num(l.lineLengthVertical)}`,
      `${k}g;${bit(l.bAllowVertScaling)}`,
      `${k}o;${num(l.lineOffset)}`,
      `${k}a;${num(l.opacity)}`,
      `${k}m;${bit(l.bShowMovementError)}`,
      `${k}f;${bit(l.bShowShootingError)}`,
    ].join(";");
  function code(p: Primary): string {
    return [
      "0;P",
      colorPart(p),
      `h;${bit(p.bHasOutline)}`,
      `t;${num(p.outlineThickness)}`,
      `o;${num(p.outlineOpacity)}`,
      `d;${bit(p.bDisplayCenterDot)}`,
      `z;${num(p.centerDotSize)}`,
      `a;${num(p.centerDotOpacity)}`,
      lines("0", p.innerLines),
      lines("1", p.outerLines),
    ].join(";");
  }
  async function copy(p: Profile) {
    await navigator.clipboard.writeText(code(p.primary));
    app.toast("success", t("configs.crosshairCopied", { name: p.profileName }));
  }

  const S = 3; // pixels d'aperçu par pixel de jeu
  const C = 48; // centre de l'aperçu (96 × 96)
  const rgba = (c: Rgba, o = 1) => `rgba(${c.r},${c.g},${c.b},${((c.a ?? 255) / 255) * o})`;

  /** Rectangles d'un viseur : point central et lignes intérieures / extérieures. */
  function rects(p: Primary) {
    const out: { x: number; y: number; w: number; h: number; o: number }[] = [];
    if (p.bDisplayCenterDot) {
      const s = p.centerDotSize * S;
      out.push({ x: C - s / 2, y: C - s / 2, w: s, h: s, o: p.centerDotOpacity });
    }
    for (const l of [p.innerLines, p.outerLines]) {
      if (!l?.bShowLines || !l.lineLength) continue;
      const th = Math.max(l.lineThickness, 1) * S;
      const len = l.lineLength * S;
      const lenV = (l.bAllowVertScaling ? l.lineLengthVertical : l.lineLength) * S;
      const off = l.lineOffset * S;
      out.push({ x: C + off, y: C - th / 2, w: len, h: th, o: l.opacity });
      out.push({ x: C - off - len, y: C - th / 2, w: len, h: th, o: l.opacity });
      out.push({ x: C - th / 2, y: C + off, w: th, h: lenV, o: l.opacity });
      out.push({ x: C - th / 2, y: C - off - lenV, w: th, h: lenV, o: l.opacity });
    }
    return out;
  }
</script>

{#if data?.profiles?.length}
  <p class="faint small">{t("configs.crosshairHint")}</p>
  <div class="grid">
    {#each data.profiles as pr, i (i)}
      {@const p = pr.primary}
      {@const color = p.bUseCustomColor ? p.colorCustom : p.color}
      <div class="card" class:on={i === data.currentProfile}>
        <button class="pick" onclick={() => choose(i)} aria-label={t("configs.crosshairPick")}>
          <svg viewBox="0 0 96 96" width="96" height="96" aria-hidden="true">
            <rect width="96" height="96" rx="10" fill="#5b6b78" />
            {#each rects(p) as r}
              {#if p.bHasOutline}
                {@const o = p.outlineThickness * S}
                <rect x={r.x - o} y={r.y - o} width={r.w + 2 * o} height={r.h + 2 * o} fill={rgba(p.outlineColor, p.outlineOpacity * r.o)} />
              {/if}
            {/each}
            {#each rects(p) as r}
              <rect x={r.x} y={r.y} width={r.w} height={r.h} fill={rgba(color, r.o)} />
            {/each}
          </svg>
          <span class="name ellipsis">{pr.profileName}</span>
        </button>
        <div class="actions">
          {#if i === data.currentProfile}<span class="chip accent">{t("configs.crosshairActive")}</span>{/if}
          <span class="spacer"></span>
          <button class="btn ghost icon sm" aria-label={t("configs.crosshairCopy")} onclick={() => copy(pr)}><Icon name="copy" size={14} /></button>
          <button class="btn ghost icon sm" aria-label={t("configs.crosshairRemove")} disabled={data.profiles.length < 2} onclick={() => remove(i)}><Icon name="trash" size={14} /></button>
        </div>
      </div>
    {/each}
  </div>
{:else}
  <p class="faint small">{t("configs.crosshairEmpty")}</p>
{/if}

<style>
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
    gap: 10px;
  }
  .card {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 10px;
    border-radius: var(--radius-md);
    background: var(--surface);
    border: 1px solid var(--border);
    min-width: 0;
  }
  .card.on {
    border-color: var(--accent);
    background: var(--accent-soft);
  }
  .pick {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    border: none;
    background: none;
    color: var(--text);
    cursor: pointer;
    min-width: 0;
  }
  .name {
    max-width: 100%;
    font-size: 12.5px;
    font-weight: 600;
  }
  .actions {
    display: flex;
    align-items: center;
    gap: 2px;
    min-height: 26px;
  }
</style>
