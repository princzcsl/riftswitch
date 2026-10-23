<script lang="ts">
  // Bouton Jouer : 1 clic = dernier jeu du compte ; ▾ = choix du jeu + en ligne / hors ligne (2 clics max).
  import type { Account, Game } from "../lib/types";
  import { app } from "../lib/app.svelte";
  import { t } from "../lib/i18n.svelte";
  import GameIcon from "./GameIcon.svelte";
  import Icon from "./Icon.svelte";
  import Popover from "./Popover.svelte";

  let { account, size = "md" }: { account: Account; size?: "md" | "lg" } = $props();
  let open = $state(false);

  const games: Game[] = ["lol", "tft", "val"];
  const defaultGame = $derived<Game>((account.lastGame ?? app.settings?.lastGame ?? "lol") as Game);
  const launching = $derived(app.launching === account.id);
  const busy = $derived(!!app.launching || !!app.status.busy);

  function play(game: Game) {
    open = false;
    app.play(account, game, app.preferOffline);
  }
</script>

<Popover bind:open width={250} align="end">
  {#snippet trigger()}
    <div class="split {size}" class:launching>
      <button class="main" disabled={busy} onclick={() => play(defaultGame)}>
        {#if launching}
          <svg class="spin" width="16" height="16" viewBox="0 0 24 24"><path d="M12 3a9 9 0 1 0 9 9" stroke="currentColor" stroke-width="3" fill="none" stroke-linecap="round" /></svg>
        {:else}
          <Icon name="play" size={size === "lg" ? 18 : 15} />
        {/if}
        <span>{launching ? t("play.launching") : t("play.play")}</span>
        <span class="g"><GameIcon game={defaultGame} size={size === "lg" ? 20 : 16} /></span>
      </button>
      <button class="caret" disabled={busy} onclick={() => (open = !open)} aria-label={t("play.chooseGame")}>
        <Icon name="chevron" size={15} />
      </button>
    </div>
  {/snippet}

  <div class="menu">
    <div class="tiles">
      {#each games as g}
        <button class="tile" class:current={g === defaultGame} onclick={() => play(g)}>
          <GameIcon game={g} size={30} />
          {#if app.settings?.playMenuLabels}<span>{t(`games.short_${g}`)}</span>{/if}
        </button>
      {/each}
    </div>
    <div class="mode">
      <button class:on={!app.preferOffline} onclick={() => app.setGhost(false)}>
        <span class="dot" style="color:var(--success)"></span>{t("play.online")}
      </button>
      <button class:on={app.preferOffline} onclick={() => app.setGhost(true)}>
        <Icon name="ghost" size={14} />{t("play.offline")}
      </button>
    </div>
  </div>
</Popover>

<style>
  .split {
    display: inline-flex;
    border-radius: var(--radius-sm);
    background: linear-gradient(135deg, var(--accent), var(--accent-2));
    box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.25);
    transition: box-shadow 0.2s;
  }
  .split:hover {
    box-shadow:
      inset 0 1px 0 rgba(255, 255, 255, 0.25),
      var(--glow);
  }
  .split button {
    border: none;
    background: transparent;
    color: var(--accent-text);
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    gap: 8px;
    font-weight: 700;
  }
  .split button:disabled {
    cursor: wait;
    opacity: 0.7;
  }
  .main {
    height: 36px;
    padding: 0 10px 0 14px;
    letter-spacing: 0.02em;
  }
  .caret {
    height: 36px;
    padding: 0 9px;
    border-left: 1px solid rgba(255, 255, 255, 0.22) !important;
  }
  .lg .main {
    height: 48px;
    padding: 0 16px 0 22px;
    font-size: 16px;
  }
  .lg .caret {
    height: 48px;
    padding: 0 13px;
  }
  .g {
    display: grid;
    filter: drop-shadow(0 1px 1px rgba(0, 0, 0, 0.3));
  }
  .launching {
    animation: glowPulse 1.2s ease-in-out infinite;
  }
  @keyframes glowPulse {
    50% {
      box-shadow: var(--glow);
    }
  }
  .menu {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .tiles {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 6px;
  }
  .tile {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    padding: 12px 4px 10px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: var(--surface);
    cursor: pointer;
    color: var(--text);
    font-size: 11.5px;
    font-weight: 650;
    transition:
      transform 0.12s var(--ease),
      background 0.15s;
  }
  .tile:hover {
    background: var(--surface-hover);
    transform: translateY(-2px);
  }
  .tile.current {
    border-color: var(--accent);
  }
  .mode {
    display: grid;
    grid-template-columns: 1fr 1fr;
    padding: 3px;
    border-radius: var(--radius-sm);
    background: var(--surface);
    border: 1px solid var(--border);
  }
  .mode button {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    height: 30px;
    border: none;
    border-radius: calc(var(--radius-sm) - 2px);
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
    font-weight: 600;
    font-size: 12.5px;
  }
  .mode button.on {
    background: var(--surface-2);
    color: var(--text);
    box-shadow: 0 1px 4px rgba(0, 0, 0, 0.2);
  }
</style>
