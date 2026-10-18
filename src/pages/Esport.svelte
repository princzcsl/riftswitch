<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { t } from "../lib/i18n.svelte";
  import GameIcon from "../components/GameIcon.svelte";
  import Icon from "../components/Icon.svelte";

  const leagues = [
    { name: "LEC", game: "lol", url: "https://lolesports.com/leagues/lec" },
    { name: "LCK", game: "lol", url: "https://lolesports.com/leagues/lck" },
    { name: "LPL", game: "lol", url: "https://lolesports.com/leagues/lpl" },
    { name: "LTA", game: "lol", url: "https://lolesports.com/leagues/lta_n" },
    { name: "VCT EMEA", game: "val", url: "https://valorantesports.com/leagues/vct_emea" },
    { name: "VCT Americas", game: "val", url: "https://valorantesports.com/leagues/vct_americas" },
    { name: "VCT Pacific", game: "val", url: "https://valorantesports.com/leagues/vct_pacific" },
    { name: "TFT", game: "tft", url: "https://teamfighttactics.leagueoflegends.com/news/esports/" },
  ] as const;
</script>

<div class="page">
  <div class="page-header">
    <div>
      <h1>{t("nav.esport")}</h1>
      <p class="muted small">{t("esport.subtitle")}</p>
    </div>
    <div class="spacer"></div>
    <span class="chip">{t("common.soon")}</span>
  </div>

  <div class="soon glass">
    <Icon name="calendar" size={22} />
    <div>
      <strong>{t("esport.soonTitle")}</strong>
      <p class="muted small">{t("esport.soonText")}</p>
    </div>
  </div>

  <div class="section-title">{t("esport.follow")}</div>
  <div class="grid">
    {#each leagues as l}
      <button class="league glass" onclick={() => openUrl(l.url)}>
        <GameIcon game={l.game} size={26} />
        <strong>{l.name}</strong>
        <span class="spacer"></span>
        <Icon name="external" size={14} />
      </button>
    {/each}
  </div>
</div>

<style>
  .soon {
    display: flex;
    gap: 14px;
    align-items: center;
    padding: 18px;
  }
  .soon :global(svg) {
    color: var(--accent);
  }
  .soon p {
    margin: 4px 0 0;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(210px, 1fr));
    gap: 10px;
  }
  .league {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 14px 16px;
    border-radius: var(--radius-md);
    cursor: pointer;
    color: var(--text);
  }
  .league:hover {
    border-color: var(--accent);
  }
</style>
