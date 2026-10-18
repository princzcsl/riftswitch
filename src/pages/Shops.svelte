<script lang="ts">
  import { app } from "../lib/app.svelte";
  import { t } from "../lib/i18n.svelte";
  import GameIcon from "../components/GameIcon.svelte";
  import Icon from "../components/Icon.svelte";
</script>

<div class="page">
  <div class="page-header">
    <div>
      <h1>{t("nav.shops")}</h1>
      <p class="muted small">{t("shops.subtitle")}</p>
    </div>
  </div>

  {#if app.settings?.streamerMode}
    <div class="notice glass"><Icon name="eyeoff" size={16} />{t("shops.streamerHidden")}</div>
  {:else}
    <div class="grid">
      <section class="shop glass">
        <div class="row">
          <GameIcon game="lol" size={28} />
          <div class="grow"><h2>{t("shops.lolTitle")}</h2><span class="risk low">● {t("risk.low")}</span></div>
          <span class="chip">{t("common.soon")}</span>
        </div>
        <p class="muted">{t("shops.lolText")}</p>
        <div class="placeholder">
          {#each Array(6) as _}<div class="tile"></div>{/each}
        </div>
      </section>

      <section class="shop glass">
        <div class="row">
          <GameIcon game="val" size={28} />
          <div class="grow"><h2>{t("shops.valTitle")}</h2><span class="risk medium">● {t("risk.medium")}</span></div>
          <span class="chip">{t("common.soon")}</span>
        </div>
        <p class="muted">{t("shops.valText")}</p>
        {#if !app.settings?.valorantStoreOptIn}
          <div class="warn">
            <Icon name="alert" size={16} />
            <div>
              <p>{t("shops.valWarn")}</p>
              <button class="btn sm" onclick={() => (app.page = "settings")}>{t("shops.enableInSettings")}</button>
            </div>
          </div>
        {:else}
          <div class="placeholder">
            {#each Array(4) as _}<div class="tile"></div>{/each}
          </div>
        {/if}
      </section>
    </div>
  {/if}
</div>

<style>
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(340px, 1fr));
    gap: 14px;
  }
  .shop {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 20px;
  }
  .placeholder {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 8px;
  }
  .tile {
    aspect-ratio: 1.3;
    border-radius: var(--radius-sm);
    background: linear-gradient(110deg, var(--surface) 30%, var(--surface-hover) 50%, var(--surface) 70%);
    background-size: 200% 100%;
    animation: shimmer 2.4s linear infinite;
    border: 1px solid var(--border);
  }
  @keyframes shimmer {
    to {
      background-position: -200% 0;
    }
  }
  .warn {
    display: flex;
    gap: 10px;
    padding: 12px;
    border-radius: var(--radius-sm);
    background: color-mix(in srgb, var(--warning) 14%, transparent);
  }
  .warn :global(svg) {
    color: var(--warning);
    flex: none;
  }
  .warn p {
    margin: 0 0 8px;
    font-size: 13px;
  }
  .notice {
    display: flex;
    gap: 10px;
    align-items: center;
    padding: 14px;
    border-radius: var(--radius-md);
  }
</style>
