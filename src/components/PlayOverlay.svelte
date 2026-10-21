<script lang="ts">
  // Overlay affiché pendant une bascule : liste les étapes franchies jusqu'au lancement.
  import { app } from "../lib/app.svelte";
  import { api } from "../lib/api";
  import { t } from "../lib/i18n.svelte";
  import Avatar from "./Avatar.svelte";
  import GameIcon from "./GameIcon.svelte";
  import Icon from "./Icon.svelte";

  const account = $derived(app.accounts.find((a) => a.id === app.launching));
  // Overlay seulement pour une vraie bascule (au moins une étape émise), pas pour un lancement direct.
  const show = $derived(!!app.launching && app.playSteps.length > 0);
  const current = $derived(app.playSteps[app.playSteps.length - 1]);
  // L'app attend une action dans le Riot Client : on peut abandonner.
  const waiting = $derived(current === "manual" || current === "restore");
</script>

{#if show && account}
  <div class="overlay">
    <div class="panel glass-strong glass-solid fade-in">
      <div class="head">
        <Avatar icon={account.icon} autoIconId={account.profileIconId} color={account.color} size={48} label={account.label} />
        <div class="col" style="gap:2px">
          <strong>{t("play.switchingTo", { name: account.label || account.gameName })}</strong>
          <span class="muted small row" style="gap:6px">
            {#if account.lastGame}<GameIcon game={account.lastGame} size={14} />{/if}
            {t("play.pleaseWait")}
          </span>
        </div>
      </div>
      <ul class="steps">
        {#each app.playSteps as step, i}
          {@const active = i === app.playSteps.length - 1}
          <li class:done={!active} class:active>
            {#if active}
              <svg class="spin" width="16" height="16" viewBox="0 0 24 24"><path d="M12 3a9 9 0 1 0 9 9" stroke="currentColor" stroke-width="3" fill="none" stroke-linecap="round" /></svg>
            {:else}
              <Icon name="check" size={16} />
            {/if}
            <span>{t(`play.step_${step}`)}</span>
          </li>
        {/each}
      </ul>
      {#if app.playSteps.includes("captcha")}
        <p class="hint small"><Icon name="info" size={13} />{t("play.captchaHint")}</p>
      {/if}
      {#if waiting}
        <div class="row" style="justify-content:flex-end;margin-top:12px">
          <button class="btn sm" onclick={() => api.cancelLogin()}>{t("common.cancel")}</button>
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .overlay {
    position: fixed;
    inset: var(--titlebar-h) 0 0 0;
    background: rgba(0, 0, 0, 0.5);
    backdrop-filter: blur(3px);
    display: grid;
    place-items: center;
    z-index: 60;
    animation: fadeIn 0.2s ease;
  }
  .panel {
    width: min(380px, calc(100vw - 48px));
    padding: 22px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  .head {
    display: flex;
    align-items: center;
    gap: 14px;
  }
  .steps {
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin: 0;
    padding: 0;
    list-style: none;
  }
  .steps li {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 10px;
    border-radius: var(--radius-sm);
    font-weight: 550;
    color: var(--text-3);
  }
  .steps li.done {
    color: var(--success);
  }
  .steps li.active {
    color: var(--text);
    background: var(--accent-soft);
  }
  .hint {
    display: flex;
    align-items: center;
    gap: 6px;
    color: var(--text-2);
    margin: 0;
  }
</style>
