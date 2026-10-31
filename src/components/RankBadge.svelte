<script lang="ts">
  // Emblème de rang original : gemme colorée par palier + libellé court.
  import type { Game, Rank } from "../lib/types";
  import { TIER_COLORS, rankShort } from "../lib/format";
  import GameIcon from "./GameIcon.svelte";
  import { t } from "../lib/i18n.svelte";

  let { rank, game, compact = false, hidden = false }: { rank: Rank | null; game: Game; compact?: boolean; hidden?: boolean } =
    $props();
  const color = $derived(rank?.tier ? (TIER_COLORS[rank.tier] ?? "#888") : "var(--text-3)");
</script>

<div class="rank" class:compact>
  <GameIcon {game} size={compact ? 14 : 16} />
  {#if hidden}
    <span class="txt faint">•••</span>
  {:else}
    <svg width="14" height="14" viewBox="0 0 16 16" aria-hidden="true">
      <path d="M8 1 14 5.5 11.7 14H4.3L2 5.5Z" fill={color} fill-opacity={rank?.tier ? 1 : 0.3} />
      <path d="M8 1 14 5.5 8 8 2 5.5Z" fill="#fff" fill-opacity=".28" />
    </svg>
    <span class="txt" style="color:{rank?.tier ? color : 'var(--text-3)'}">{rankShort(rank)}</span>
    {#if !compact && rank?.tier}<span class="lp">{rank.lp} LP</span>{/if}
  {/if}
</div>

<style>
  .rank {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 26px;
    padding: 0 9px 0 6px;
    border-radius: var(--radius-pill);
    background: color-mix(in srgb, var(--surface-hover) 80%, transparent);
    border: 1px solid var(--border);
    font-size: 12px;
    font-weight: 650;
    white-space: nowrap;
  }
  .rank.compact {
    height: 22px;
    font-size: 11.5px;
    padding: 0 7px 0 5px;
    gap: 5px;
  }
  .lp {
    color: var(--text-3);
    font-weight: 600;
  }
</style>
