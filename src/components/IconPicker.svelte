<script lang="ts">
  // Choix de l'icône du compte : set original, champions (Data Dragon), icône d'invocateur, ou image importée.
  import { GLYPHS } from "../lib/avatars";
  import { champions, championIcon, ddLang, valorantAgents, type Agent, type ChampionLite } from "../lib/ddragon";
  import { app } from "../lib/app.svelte";
  import { i18n, t } from "../lib/i18n.svelte";
  import Avatar from "./Avatar.svelte";
  import CropTool from "./CropTool.svelte";

  let { value = $bindable(""), color, profileIconId }: { value?: string; color: string; profileIconId: number | null } = $props();

  let tab = $state<"set" | "champ" | "agent" | "import">("set");
  let list = $state<ChampionLite[]>([]);
  let agents = $state<Agent[]>([]);
  let q = $state("");
  let cropSrc = $state("");
  let fileInput = $state<HTMLInputElement>();

  $effect(() => {
    if (tab === "champ" && !list.length) champions(ddLang(i18n.lang)).then((l) => (list = l));
    if (tab === "agent" && !agents.length) valorantAgents().then((a) => (agents = a));
  });

  const filtered = $derived(q ? list.filter((c) => c.name.toLowerCase().includes(q.toLowerCase())) : list);
  const filteredAgents = $derived(q ? agents.filter((a) => a.name.toLowerCase().includes(q.toLowerCase())) : agents);

  function onFile(e: Event) {
    const f = (e.currentTarget as HTMLInputElement).files?.[0];
    if (!f) return;
    const reader = new FileReader();
    reader.onload = () => (cropSrc = String(reader.result));
    reader.readAsDataURL(f);
  }
</script>

<div class="picker">
  <div class="tabs">
    <button class:on={tab === "set"} onclick={() => (tab = "set")}>{t("icon.set")}</button>
    <button class:on={tab === "champ"} onclick={() => (tab = "champ")}>{t("icon.champions")}</button>
    <button class:on={tab === "agent"} onclick={() => (tab = "agent")}>{t("icon.agents")}</button>
    <button class:on={tab === "import"} onclick={() => (tab = "import")}>{t("icon.import")}</button>
  </div>

  {#if tab === "set"}
    <div class="grid">
      <button class="cell auto" class:sel={value === "auto" || !value} onclick={() => (value = "auto")} aria-label={t("icon.autoHint")}>
        <div class="auto-badge" style="--c:{color}">{t("icon.autoShort")}</div>
      </button>
      {#if profileIconId}
        <button class="cell" class:sel={value === `profile:${profileIconId}`} onclick={() => (value = `profile:${profileIconId}`)} aria-label={t("icon.summoner")}>
          <Avatar icon={`profile:${profileIconId}`} {color} size={46} />
        </button>
      {/if}
      {#each GLYPHS as g}
        <button class="cell" class:sel={value === `glyph:${g.id}`} onclick={() => (value = `glyph:${g.id}`)}>
          <Avatar icon={`glyph:${g.id}`} {color} size={46} />
        </button>
      {/each}
    </div>
    <p class="faint small">{t("icon.autoHint")}</p>
  {:else if tab === "champ"}
    <input class="input" placeholder={t("common.search")} bind:value={q} />
    <div class="grid champs">
      {#each filtered as c (c.id)}
        <button class="cell" class:sel={value === `champ:${c.id}`} onclick={() => (value = `champ:${c.id}`)} aria-label={c.name}>
          {#if app.ddVersion}<img src={championIcon(app.ddVersion, c.id)} alt={c.name} loading="lazy" />{/if}
        </button>
      {:else}
        <p class="faint small">{t("icon.loading")}</p>
      {/each}
    </div>
    <p class="faint small">{t("icon.legal")}</p>
  {:else if tab === "agent"}
    <input class="input" placeholder={t("common.search")} bind:value={q} />
    <div class="grid champs">
      {#each filteredAgents as a (a.uuid)}
        <button class="cell" class:sel={value === `agent:${a.uuid}`} onclick={() => (value = `agent:${a.uuid}`)} aria-label={a.name}>
          <img src={a.icon} alt={a.name} loading="lazy" />
        </button>
      {:else}
        <p class="faint small">{t("icon.loading")}</p>
      {/each}
    </div>
    <p class="faint small">{t("icon.legal")}</p>
  {:else if cropSrc}
    <CropTool
      src={cropSrc}
      oncancel={() => (cropSrc = "")}
      ondone={(url) => {
        value = url;
        cropSrc = "";
        tab = "set";
      }}
    />
  {:else}
    <div class="drop col">
      <p class="muted">{t("icon.importHint")}</p>
      <button class="btn" onclick={() => fileInput?.click()}>{t("icon.chooseImage")}</button>
      <input bind:this={fileInput} type="file" accept="image/png,image/jpeg,image/webp,image/gif" hidden onchange={onFile} />
    </div>
  {/if}
</div>

<style>
  .picker {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .tabs {
    display: flex;
    gap: 4px;
  }
  .tabs button {
    border: 1px solid var(--border);
    background: transparent;
    color: var(--text-2);
    padding: 6px 12px;
    border-radius: var(--radius-pill);
    cursor: pointer;
    font-weight: 600;
    font-size: 12.5px;
  }
  .tabs button.on {
    background: var(--accent-soft);
    color: var(--accent);
    border-color: transparent;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(54px, 1fr));
    gap: 6px;
  }
  .champs {
    max-height: 220px;
    overflow-y: auto;
  }
  .cell {
    display: grid;
    place-items: center;
    padding: 4px;
    border-radius: var(--radius-sm);
    border: 2px solid transparent;
    background: transparent;
    cursor: pointer;
  }
  .cell:hover {
    background: var(--surface-hover);
  }
  .cell.sel {
    border-color: var(--accent);
  }
  .cell img {
    width: 46px;
    height: 46px;
    border-radius: 50%;
    object-fit: cover;
  }
  .auto-badge {
    width: 46px;
    height: 46px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    font-size: 10px;
    font-weight: 700;
    color: var(--c);
    border: 2px dashed color-mix(in srgb, var(--c) 60%, var(--border));
    background: color-mix(in srgb, var(--c) 12%, transparent);
  }
  .drop {
    align-items: center;
    padding: 24px;
    border: 1.5px dashed var(--border);
    border-radius: var(--radius-md);
    text-align: center;
  }
</style>
