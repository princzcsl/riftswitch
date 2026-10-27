<script lang="ts">
  // Profils de viseur VALORANT (SavedCrosshairProfileData) : aperçu de chaque viseur et choix de l'actif.
  import type { IniDoc } from "../lib/types";
  import { t } from "../lib/i18n.svelte";
  import { bareKey } from "../lib/settingsSchema";

  type Rgba = { r: number; g: number; b: number; a: number };
  type Lines = { lineThickness: number; lineLength: number; lineLengthVertical: number; lineOffset: number; bAllowVertScaling: boolean; opacity: number; bShowLines: boolean };
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

  function choose(i: number) {
    const p = pos();
    if (!p || !data) return;
    doc.sections[p.si].entries[p.ei][1] = JSON.stringify(JSON.stringify({ ...data, currentProfile: i }));
    onchange?.();
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
    {#each data.profiles as pr, i}
      {@const p = pr.primary}
      {@const color = p.bUseCustomColor ? p.colorCustom : p.color}
      <button class="card" class:on={i === data.currentProfile} onclick={() => choose(i)}>
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
        <span class="name ellipsis" title={pr.profileName}>{pr.profileName}</span>
        {#if i === data.currentProfile}<span class="chip accent">{t("configs.crosshairActive")}</span>{/if}
      </button>
    {/each}
  </div>
{:else}
  <p class="faint small">{t("configs.crosshairEmpty")}</p>
{/if}

<style>
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: 10px;
  }
  .card {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 12px 10px;
    border-radius: var(--radius-md);
    background: var(--surface);
    border: 1px solid var(--border);
    color: var(--text);
    cursor: pointer;
    min-width: 0;
  }
  .card.on {
    border-color: var(--accent);
    background: var(--accent-soft);
  }
  .name {
    max-width: 100%;
    font-size: 12.5px;
    font-weight: 600;
  }
</style>
