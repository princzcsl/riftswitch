<script lang="ts">
  // Bandeau « Reprendre » : le dernier compte joué, en grand, avec le bouton Jouer bien visible.
  import type { Account } from "../lib/types";
  import { app } from "../lib/app.svelte";
  import { t } from "../lib/i18n.svelte";
  import { displayRiotId, hours, recentPlaytime } from "../lib/format";
  import Avatar from "./Avatar.svelte";
  import PlayButton from "./PlayButton.svelte";
  import RankBadge from "./RankBadge.svelte";

  let { account }: { account: Account } = $props();
  const streamer = $derived(app.settings?.streamerMode ?? false);
  const connected = $derived(app.status.activeAccountId === account.id);
</script>

<section class="hero glass" style="--c:{account.color}">
  <div class="clip"><div class="glow"></div></div>
  <Avatar icon={account.icon} autoIconId={account.profileIconId} color={account.color} size={84} label={account.label} ring={connected} />
  <div class="info grow">
    <div class="eyebrow">{connected ? t("hero.connected") : t("hero.resume")}</div>
    <h1 class="ellipsis">{account.label || account.gameName || t("accounts.unnamed")}</h1>
    <div class="muted ellipsis">
      {displayRiotId(account, streamer)}{#if account.region}&nbsp;· {account.region}{/if}
      {#if account.level && !streamer}&nbsp;· {t("hero.level", { n: account.level })}{/if}
    </div>
    <div class="ranks">
      <RankBadge game="lol" rank={account.ranks.lolSolo} hidden={streamer} />
      <RankBadge game="tft" rank={account.ranks.tft} hidden={streamer} />
      <RankBadge game="val" rank={account.ranks.val} hidden={streamer} />
    </div>
  </div>
  <div class="side">
    <div class="stat">
      <span class="faint small">{t("hero.week")}</span>
      <strong>{hours(recentPlaytime(account, 7))}</strong>
    </div>
    <PlayButton {account} size="lg" />
  </div>
</section>

<style>
  .hero {
    position: relative;
    display: flex;
    align-items: center;
    gap: 22px;
    padding: 22px 24px;
    margin-bottom: 8px;
    z-index: 5;
  }
  .clip {
    position: absolute;
    inset: 0;
    border-radius: inherit;
    overflow: hidden;
    pointer-events: none;
  }
  .glow {
    position: absolute;
    inset: -40% 40% -40% -10%;
    background: radial-gradient(closest-side, color-mix(in srgb, var(--c) 38%, transparent), transparent);
    pointer-events: none;
  }
  .info {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .eyebrow {
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--accent);
  }
  h1 {
    font-size: 28px;
  }
  .ranks {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-top: 8px;
  }
  .side {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 12px;
  }
  .stat {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
  }
  .stat strong {
    font-size: 20px;
  }
</style>
