<script lang="ts">
  // Pictogrammes originaux évoquant chaque jeu (pas les logos officiels).
  import type { Game } from "../lib/types";
  let { game, size = 22 }: { game: Game; size?: number } = $props();
</script>

<svg width={size} height={size} viewBox="0 0 32 32" aria-hidden="true" style="flex:none">
  {#if game === "lol"}
    <defs>
      <linearGradient id="gi-lol" x1="0" y1="0" x2="0" y2="1">
        <stop offset="0" stop-color="#f0d9a0" />
        <stop offset="1" stop-color="#b08a43" />
      </linearGradient>
    </defs>
    <path d="M16 2 28 9v14l-12 7-12-7V9Z" fill="#0a1428" stroke="url(#gi-lol)" stroke-width="2" />
    <path d="M11 9h4v11h7l-2 3H11Z" fill="url(#gi-lol)" />
  {:else if game === "tft"}
    <defs>
      <linearGradient id="gi-tft" x1="0" y1="0" x2="1" y2="1">
        <stop offset="0" stop-color="#ffd27a" />
        <stop offset="1" stop-color="#d07b2a" />
      </linearGradient>
    </defs>
    <path d="M16 2 28 9v14l-12 7-12-7V9Z" fill="#1c1030" stroke="url(#gi-tft)" stroke-width="2" />
    <path d="M9 21l1.5-9 3.5 4 2-6 2 6 3.5-4 1.5 9Z" fill="url(#gi-tft)" />
  {:else}
    <rect x="2" y="2" width="28" height="28" rx="6" fill="#ff4655" />
    <path d="M7 8l7.5 12V24L7 12Z M25 8l-6 9.5h-4.5L21 8Z" fill="#fff" />
  {/if}
</svg>
