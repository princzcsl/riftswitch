<script lang="ts">
  import type { Game } from "../lib/types";
  import { app } from "../lib/app.svelte";
  import { t } from "../lib/i18n.svelte";
  import { TIERS, displayRiotId } from "../lib/format";
  import { suggest, type Goal } from "../lib/tonight";
  import Avatar from "../components/Avatar.svelte";
  import GameIcon from "../components/GameIcon.svelte";
  import Icon from "../components/Icon.svelte";
  import PlayButton from "../components/PlayButton.svelte";
  import RankBadge from "../components/RankBadge.svelte";

  let goal = $state<Goal>("grind");
  let game = $state<Game>((app.settings?.lastGame as Game) ?? "lol");
  let role = $state("jungle");
  // Garde un rôle cohérent avec le jeu choisi.
  $effect(() => {
    const valo = ["duelist", "controller", "initiator", "sentinel"];
    if (game === "val" && !valo.includes(role)) role = "duelist";
    else if (game !== "val" && valo.includes(role)) role = "jungle";
  });
  let friendTier = $state("GOLD");

  const goals: { id: Goal; icon: string }[] = [
    { id: "grind", icon: "flame" },
    { id: "role", icon: "target" },
    { id: "friends", icon: "users" },
    { id: "decay", icon: "calendar" },
  ];

  const results = $derived(suggest(app.accounts, goal, { game, role, friendTier }).slice(0, 5));
  const streamer = $derived(app.settings?.streamerMode ?? false);
</script>

<div class="page">
  <div class="page-header">
    <div>
      <h1>{t("tonight.title")}</h1>
      <p class="muted small">{t("tonight.subtitle")}</p>
    </div>
  </div>

  <div class="goals">
    {#each goals as g}
      <button class="goal glass" class:on={goal === g.id} onclick={() => (goal = g.id)}>
        <Icon name={g.icon} size={24} />
        <strong>{t(`tonight.goal_${g.id}`)}</strong>
        <span class="small muted">{t(`tonight.goal_${g.id}_text`)}</span>
      </button>
    {/each}
  </div>

  <div class="filters row">
    <div class="seg">
      {#each ["lol", "tft", "val"] as g}
        <button class:on={game === g} onclick={() => (game = g as Game)}><GameIcon game={g as Game} size={16} />{t(`games.short_${g}`)}</button>
      {/each}
    </div>
    {#if goal === "role"}
      <select class="input" bind:value={role}>
        {#each game === "val" ? ["duelist", "controller", "initiator", "sentinel"] : ["top", "jungle", "mid", "adc", "support"] as r}<option value={r}>{t(`roles.${r}`)}</option>{/each}
      </select>
    {:else if goal === "friends"}
      <span class="muted small">{t("tonight.friendsRank")}</span>
      <select class="input" bind:value={friendTier}>
        {#each TIERS as tier}<option value={tier}>{t(`tiers.${tier.toLowerCase()}`)}</option>{/each}
      </select>
    {/if}
  </div>

  {#if !app.accounts.length}
    <p class="empty">{t("tonight.noAccounts")}</p>
  {:else}
    <div class="results">
      {#each results as r, i (r.account.id)}
        <div class="result glass" class:best={i === 0}>
          {#if i === 0}<span class="crown">{t("tonight.best")}</span>{/if}
          <Avatar icon={r.account.icon} autoIconId={r.account.profileIconId} color={r.account.color} size={i === 0 ? 64 : 46} label={r.account.label} />
          <div class="grow col" style="gap:6px">
            <div class="row">
              <strong class="ellipsis">{r.account.label || r.account.gameName || t("accounts.unnamed")}</strong>
              <span class="faint small ellipsis">{displayRiotId(r.account, streamer)}</span>
            </div>
            <div class="reasons">
              {#each r.reasons as reason}
                <span class="chip {reason.good ? 'success' : 'warning'}">{t(reason.key, reason.params)}</span>
              {/each}
            </div>
          </div>
          <RankBadge {game} rank={game === "val" ? r.account.ranks.val : game === "tft" ? r.account.ranks.tft : r.account.ranks.lolSolo} hidden={streamer} />
          <PlayButton account={r.account} />
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .goals {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
    gap: 10px;
  }
  .goal {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 6px;
    padding: 16px;
    cursor: pointer;
    text-align: start;
    color: var(--text);
    transition: transform 0.15s var(--ease);
  }
  .goal:hover {
    transform: translateY(-2px);
  }
  .goal.on {
    border-color: var(--accent);
    background: var(--accent-soft);
  }
  .goal :global(svg) {
    color: var(--accent);
  }
  .filters {
    margin: 18px 0 12px;
  }
  .filters select {
    width: 180px;
  }
  .seg {
    display: flex;
    padding: 3px;
    border-radius: var(--radius-sm);
    background: var(--surface);
    border: 1px solid var(--border);
  }
  .seg button {
    display: flex;
    align-items: center;
    gap: 6px;
    height: 30px;
    padding: 0 12px;
    border: none;
    border-radius: calc(var(--radius-sm) - 2px);
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
    font-weight: 600;
  }
  .seg button.on {
    background: var(--surface-2);
    color: var(--text);
  }
  .results {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .result {
    position: relative;
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 14px 16px;
  }
  .result.best {
    padding: 20px;
    border-color: color-mix(in srgb, var(--accent) 60%, var(--border));
    box-shadow:
      inset 0 1px 0 var(--highlight),
      var(--glow);
  }
  .crown {
    position: absolute;
    top: -10px;
    left: 18px;
    padding: 2px 10px;
    border-radius: 999px;
    background: linear-gradient(135deg, var(--accent), var(--accent-2));
    color: var(--accent-text);
    font-size: 11px;
    font-weight: 700;
  }
  .reasons {
    display: flex;
    flex-wrap: wrap;
    gap: 5px;
  }
</style>
