<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { allVersions } from "../lib/ddragon";
  import { i18n, t } from "../lib/i18n.svelte";
  import GameIcon from "../components/GameIcon.svelte";
  import Icon from "../components/Icon.svelte";

  let versions = $state<string[]>([]);
  let seen = $state("");

  $effect(() => {
    try {
      seen = localStorage.getItem("rs.patchSeen") ?? "";
    } catch {
      seen = "";
    }
    allVersions().then((v) => {
      versions = v.filter((x) => /^\d+\.\d+\.\d+$/.test(x)).slice(0, 6);
      if (versions[0]) {
        try {
          localStorage.setItem("rs.patchSeen", versions[0]);
        } catch {
          /* ignoré */
        }
      }
    });
  });

  /** Version Data Dragon 15.20.1 → patch affiché 25.20 (numérotation par année depuis 2025). */
  function patchName(v: string) {
    const [maj, min] = v.split(".").map(Number);
    return maj >= 15 ? `${maj + 10}.${min}` : `${maj}.${min}`;
  }

  const locale = $derived(
    ({ fr: "fr-fr", es: "es-es", it: "it-it", de: "de-de", "pt-BR": "pt-br", pl: "pl-pl", tr: "tr-tr", ru: "ru-ru", ko: "ko-kr", ja: "ja-jp" } as Record<string, string>)[
      i18n.lang
    ] ?? "en-us",
  );

  function notesUrl(v: string) {
    return `https://www.leagueoflegends.com/${locale}/news/game-updates/patch-${patchName(v).replace(".", "-")}-notes/`;
  }
</script>

<div class="page">
  <div class="page-header">
    <div>
      <h1>{t("nav.patches")}</h1>
      <p class="muted small">{t("patches.subtitle")}</p>
    </div>
  </div>

  {#if versions[0]}
    <section class="current glass">
      <GameIcon game="lol" size={44} />
      <div class="grow">
        <div class="eyebrow">{t("patches.current")}</div>
        <h2>{t("patches.patch", { v: patchName(versions[0]) })}</h2>
        {#if seen && seen !== versions[0]}<span class="chip accent">{t("patches.new")}</span>{/if}
      </div>
      <button class="btn primary" onclick={() => openUrl(notesUrl(versions[0]))}><Icon name="external" size={15} />{t("patches.readNotes")}</button>
    </section>
  {/if}

  <div class="grid">
    <div class="card glass">
      <div class="row"><GameIcon game="tft" /><strong>Teamfight Tactics</strong></div>
      <p class="muted small">{t("patches.tftText")}</p>
      <button class="btn sm" onclick={() => openUrl("https://teamfighttactics.leagueoflegends.com/news/game-updates/")}><Icon name="external" size={14} />{t("patches.open")}</button>
    </div>
    <div class="card glass">
      <div class="row"><GameIcon game="val" /><strong>VALORANT</strong></div>
      <p class="muted small">{t("patches.valText")}</p>
      <button class="btn sm" onclick={() => openUrl("https://playvalorant.com/news/game-updates/")}><Icon name="external" size={14} />{t("patches.open")}</button>
    </div>
    <div class="card glass">
      <div class="row"><Icon name="sparkle" /><strong>{t("patches.recapTitle")}</strong><span class="chip">{t("common.soon")}</span></div>
      <p class="muted small">{t("patches.recapText")}</p>
    </div>
  </div>

  {#if versions.length > 1}
    <div class="section-title">{t("patches.history")}</div>
    <div class="history">
      {#each versions.slice(1) as v}
        <button class="hist glass" onclick={() => openUrl(notesUrl(v))}>
          <span>{t("patches.patch", { v: patchName(v) })}</span><span class="faint small mono">{v}</span><Icon name="external" size={14} />
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .current {
    display: flex;
    align-items: center;
    gap: 18px;
    padding: 20px 22px;
    margin-bottom: 14px;
  }
  .eyebrow {
    font-size: 11px;
    font-weight: 700;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--accent);
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
    gap: 10px;
  }
  .card {
    display: flex;
    flex-direction: column;
    gap: 10px;
    align-items: flex-start;
  }
  .history {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .hist {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 14px;
    border-radius: var(--radius-sm);
    cursor: pointer;
    color: var(--text);
    justify-content: space-between;
  }
</style>
