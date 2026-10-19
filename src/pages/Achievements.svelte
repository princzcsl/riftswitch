<script lang="ts">
  import { app } from "../lib/app.svelte";
  import { t } from "../lib/i18n.svelte";
  import { computeAchievements, shards } from "../lib/achievements";
  import { THEMES } from "../lib/themes";

  const list = $derived(computeAchievements(app.accounts, app.folders, app.profiles));
  const balance = $derived(shards(list) + (app.settings?.bonusShards ?? 0));
  const unlockables = $derived(THEMES.filter((th) => th.cost > 0));
</script>

<div class="page">
  <div class="page-header">
    <div>
      <h1>{t("nav.achievements")}</h1>
      <p class="muted small">{t("achievements.subtitle")}</p>
    </div>
    <div class="spacer"></div>
    <div class="balance glass"><span class="shard">✦</span><strong>{balance}</strong><span class="muted small">{t("achievements.shards")}</span></div>
  </div>

  <div class="grid">
    {#each list as a (a.id)}
      {@const done = a.progress >= a.goal}
      <div class="ach glass" class:done>
        <div class="icon">{a.icon}</div>
        <div class="grow col" style="gap:6px">
          <div class="row">
            <strong class="grow">{t(`achievements.${a.id}`)}</strong>
            <span class="reward">+{a.reward} ✦</span>
          </div>
          <span class="muted small">{t(`achievements.${a.id}_text`)}</span>
          <div class="bar"><span style="width:{Math.min(100, (a.progress / a.goal) * 100)}%"></span></div>
          <span class="faint small">{a.progress} / {a.goal}</span>
        </div>
      </div>
    {/each}
  </div>

  <div class="section-title">{t("achievements.rewards")}</div>
  <div class="grid">
    {#each unlockables as th}
      {@const ok = balance >= th.cost}
      <div class="reward-card glass" style="--a:{th.tokens.accent};--b:{th.tokens['accent-2']};--bg:{th.tokens['bg-solid']}">
        <span class="sw"></span>
        <div class="grow">
          <strong>{th.name}</strong>
          <div class="small muted">{th.cost} ✦ · {ok ? t("achievements.unlocked") : t("achievements.locked")}</div>
        </div>
        {#if ok}<button class="btn sm" onclick={() => app.saveSettings({ theme: th.id })}>{t("achievements.apply")}</button>{/if}
      </div>
    {/each}
  </div>
  <p class="faint small" style="margin-top:12px">{t("achievements.noMoney")}</p>
</div>

<style>
  .balance {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 16px;
    border-radius: var(--radius-pill);
  }
  .shard {
    color: var(--accent);
    font-size: 18px;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 10px;
  }
  .ach {
    display: flex;
    gap: 14px;
    padding: 16px;
    opacity: 0.85;
  }
  .ach.done {
    opacity: 1;
    border-color: color-mix(in srgb, var(--accent) 50%, var(--border));
  }
  .icon {
    width: 46px;
    height: 46px;
    flex: none;
    border-radius: 12px;
    display: grid;
    place-items: center;
    font-size: 22px;
    background: var(--surface-hover);
    filter: grayscale(1);
  }
  .done .icon {
    background: linear-gradient(135deg, var(--accent), var(--accent-2));
    color: var(--accent-text);
    filter: none;
  }
  .reward {
    font-size: 12px;
    font-weight: 700;
    color: var(--accent);
  }
  .bar {
    height: 5px;
    border-radius: 5px;
    background: var(--border);
    overflow: hidden;
  }
  .bar span {
    display: block;
    height: 100%;
    background: linear-gradient(90deg, var(--accent), var(--accent-2));
  }
  .reward-card {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 14px;
  }
  .sw {
    width: 40px;
    height: 40px;
    border-radius: 12px;
    background: linear-gradient(135deg, var(--a), var(--b));
    box-shadow: 0 0 0 3px var(--bg);
    flex: none;
  }
</style>
