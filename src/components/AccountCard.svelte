<script lang="ts">
  import type { Account, Game, Rank } from "../lib/types";
  import { app } from "../lib/app.svelte";
  import { t } from "../lib/i18n.svelte";
  import { displayRiotId, hours, relativeTime } from "../lib/format";
  import Avatar from "./Avatar.svelte";
  import Icon from "./Icon.svelte";
  import PlayButton from "./PlayButton.svelte";
  import Popover from "./Popover.svelte";
  import RankBadge from "./RankBadge.svelte";

  let {
    account,
    showFolder = false,
    onedit,
    ondelete,
    onfavorite,
    oncontext,
  }: {
    account: Account;
    showFolder?: boolean;
    onedit: () => void;
    ondelete: () => void;
    onfavorite: () => void;
    oncontext?: (e: MouseEvent) => void;
  } = $props();

  let menu = $state(false);
  const streamer = $derived(app.settings?.streamerMode ?? false);
  const connected = $derived(app.status.activeAccountId === account.id);
  const folder = $derived(app.folders.find((f) => f.id === account.folderId));
  const gameTime = $derived(account.playtime.lol + account.playtime.tft + account.playtime.val);
  const launcherTime = $derived((account.playtime.launcher?.lol ?? 0) + (account.playtime.launcher?.tft ?? 0) + (account.playtime.launcher?.val ?? 0));
  const decay = $derived(account.decay.daysRemaining);
  // Seulement les jeux classés, pour garder la carte sur une ligne.
  const ranked = $derived(
    (
      [
        ["lol", account.ranks.lolSolo],
        ["tft", account.ranks.tft],
        ["val", account.ranks.val],
      ] as [Game, Rank | null][]
    ).filter(([, r]) => r?.tier),
  );
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<article
  class="card glass fade-in"
  class:connected
  style="--c:{account.color}"
  oncontextmenu={(e) => {
    if (oncontext) {
      e.preventDefault();
      oncontext(e);
    }
  }}
>
  <div class="band"></div>
  <header>
    <Avatar icon={account.icon} autoIconId={account.profileIconId} color={account.color} size={54} label={account.label} ring={connected} />
    <div class="who grow">
      <div class="row name-row">
        <h3 class="ellipsis">{account.label || account.gameName || t("accounts.unnamed")}</h3>
      </div>
      <div class="riot ellipsis">
        {#if account.riotId}{displayRiotId(account, streamer)}{:else}<span class="faint">{t("accounts.noRiotId")}</span>{/if}
      </div>
    </div>
    <button class="btn ghost icon sm fav" class:on={account.favorite} onclick={onfavorite}>
      <Icon name={account.favorite ? "starFill" : "star"} size={17} />
    </button>
    <Popover bind:open={menu} width={230}>
      {#snippet trigger()}
        <button class="btn ghost icon sm" onclick={() => (menu = !menu)} aria-label={t("common.more")}><Icon name="more" /></button>
      {/snippet}
      <button class="menu-item" onclick={() => ((menu = false), onedit())}><Icon name="edit" size={16} />{t("common.edit")}</button>
      {#if account.riotId && !streamer}
        <button
          class="menu-item"
          onclick={() => {
            menu = false;
            navigator.clipboard.writeText(account.riotId);
            app.toast("success", t("common.copied"));
          }}><Icon name="copy" size={16} />{t("accounts.copyRiotId")}</button
        >
      {/if}
      <div class="divider"></div>
      <button class="menu-item danger" onclick={() => ((menu = false), ondelete())}><Icon name="trash" size={16} />{t("common.delete")}</button>
    </Popover>
  </header>

  <div class="chips">
    {#if connected}<span class="chip success"><span class="dot pulse"></span>{t("accounts.connected")}</span>{/if}
    {#if account.region}<span class="chip">{account.region}</span>{/if}
    {#if account.valShard}<span class="chip">VAL {account.valShard.toUpperCase()}</span>{/if}
    {#if showFolder && folder}<span class="chip" style="color:{folder.color || 'inherit'}"><Icon name="folder" size={12} />{folder.name}</span>{/if}
    {#each account.tags.slice(0, 3) as tag}<span class="chip accent">{tag}</span>{/each}
  </div>

  <div class="ranks">
    {#each ranked as [g, r]}
      <RankBadge game={g} rank={r} compact hidden={streamer} />
    {:else}
      <RankBadge game={(account.lastGame ?? "lol") as Game} rank={null} compact />
    {/each}
  </div>

  {#if !account.hasSession || decay != null || account.health.penalties.length || !account.twoFa}
    <div class="alerts">
      {#if !account.hasSession}<span class="chip danger"><Icon name="alert" size={12} />{t("accounts.noSession")}</span>{/if}
      {#if decay != null}<span class="chip warning"><Icon name="flame" size={12} />{t("accounts.decayIn", { n: decay })}</span>{/if}
      {#if account.health.penalties.length}<span class="chip danger">{t("accounts.penalty")}</span>{/if}
      {#if !account.twoFa}<span class="chip warning"><Icon name="shield" size={12} />{t("accounts.no2fa")}</span>{/if}
    </div>
  {/if}

  <footer>
    <div class="meta small faint">
      <span>⏱ {hours(gameTime + launcherTime)}</span>
      <span>· {relativeTime(account.lastUsedAt)}</span>
    </div>
    <PlayButton {account} />
  </footer>
</article>

<style>
  .card {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 16px;
    transition:
      transform 0.18s var(--ease),
      border-color 0.2s,
      box-shadow 0.2s;
  }
  .card:hover {
    transform: translateY(-3px);
    border-color: color-mix(in srgb, var(--c) 50%, var(--border));
    box-shadow:
      inset 0 1px 0 var(--highlight),
      0 14px 34px color-mix(in srgb, var(--c) 18%, transparent);
  }
  /* Un menu ouvert doit passer au-dessus des cartes voisines. */
  .card:has(:global(.pop)) {
    z-index: 10;
  }
  .card.connected {
    border-color: color-mix(in srgb, var(--success) 45%, var(--border));
  }
  .band {
    position: absolute;
    inset: 0 0 auto 0;
    height: 70px;
    border-radius: var(--radius-lg) var(--radius-lg) 0 0;
    background: radial-gradient(120% 100% at 0% 0%, color-mix(in srgb, var(--c) 30%, transparent), transparent 70%);
    pointer-events: none;
  }
  header {
    position: relative;
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .who {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  h3 {
    font-size: 15.5px;
  }
  .riot {
    font-size: 12.5px;
    color: var(--text-2);
  }
  .fav {
    color: var(--text-3);
  }
  .fav.on {
    color: #ffc94d;
  }
  .chips,
  .ranks,
  .alerts {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    position: relative;
  }
  .chips:empty {
    display: none;
  }
  footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    padding-top: 12px;
    margin-top: auto;
    border-top: 1px solid var(--border);
  }
  .meta {
    display: flex;
    gap: 4px;
    white-space: nowrap;
  }
</style>
