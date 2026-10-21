<script lang="ts">
  import type { RunePage } from "../lib/types";
  import { api } from "../lib/api";
  import { app } from "../lib/app.svelte";
  import { errorText, i18n, t } from "../lib/i18n.svelte";
  import { ddLang, runeIcon, runeStyles, type RuneStyle } from "../lib/ddragon";
  import Icon from "../components/Icon.svelte";
  import Modal from "../components/Modal.svelte";

  let styles = $state<RuneStyle[]>([]);
  let clientPages = $state<RunePage[]>([]);
  let library = $state<RunePage[]>([]);
  let clientError = $state("");
  let loadingClient = $state(false);
  let importOpen = $state(false);
  let code = $state("");

  $effect(() => {
    runeStyles(ddLang(i18n.lang)).then((s) => (styles = s));
  });

  $effect(() => {
    loadLibrary();
    loadClient();
  });

  async function loadLibrary() {
    library = await api.runesLibrary().catch(() => []);
  }

  async function loadClient() {
    loadingClient = true;
    clientError = "";
    try {
      clientPages = await api.runesClientPages();
    } catch (e) {
      clientPages = [];
      clientError = errorText(e);
    } finally {
      loadingClient = false;
    }
  }

  function perk(id: number) {
    for (const s of styles) for (const slot of s.slots) for (const r of slot.runes) if (r.id === id) return r;
    return null;
  }
  function style(id: number) {
    return styles.find((s) => s.id === id);
  }

  async function save(p: RunePage) {
    const active = app.activeAccount;
    await api.runesSave({ ...p, id: "", sourceAccount: active?.id ?? null });
    app.toast("success", t("runes.saved"));
    loadLibrary();
  }

  async function push(p: RunePage) {
    try {
      await api.runesPush(p);
      app.toast("success", t("runes.pushed"));
      loadClient();
    } catch (e) {
      app.toast("error", errorText(e));
    }
  }

  async function share(p: RunePage) {
    const c = await api.runesExportCode(p);
    await navigator.clipboard.writeText(c);
    app.toast("success", t("runes.codeCopied"));
  }

  async function doImport() {
    try {
      await api.runesImportCode(code);
      importOpen = false;
      code = "";
      loadLibrary();
      app.toast("success", t("runes.imported"));
    } catch (e) {
      app.toast("error", errorText(e));
    }
  }

  async function remove(p: RunePage) {
    await api.runesDelete(p.id);
    loadLibrary();
  }
</script>

{#snippet pageCard(p: RunePage, kind: "client" | "lib")}
  {@const ps = style(p.primaryStyleId)}
  {@const ss = style(p.subStyleId)}
  <div class="rune glass">
    <div class="row">
      {#if ps}<img class="style" src={runeIcon(ps.icon)} alt={ps.name} />{/if}
      <strong class="grow ellipsis">{p.name}</strong>
      {#if ss}<img class="style small-s" src={runeIcon(ss.icon)} alt={ss.name} />{/if}
    </div>
    {#if kind === "lib" && p.sourceAccount}
      {@const src = app.accounts.find((a) => a.id === p.sourceAccount)}
      {#if src}
        <span class="source faint small" title={t("runes.source")}>
          <span class="dot" style="background:{src.color}"></span>{src.label || src.gameName}
        </span>
      {/if}
    {/if}
    <div class="perks">
      {#each p.selectedPerkIds.slice(0, 6) as id, i}
        {@const r = perk(id)}
        {#if r}<img class="perk" class:key={i === 0} src={runeIcon(r.icon)} alt={r.name} title={r.name} />{:else}<span class="perk ph"></span>{/if}
      {/each}
    </div>
    <div class="row actions">
      {#if kind === "client"}
        <button class="btn sm" onclick={() => save(p)}><Icon name="download" size={14} />{t("runes.save")}</button>
      {:else}
        <button class="btn sm primary" onclick={() => push(p)} disabled={!!clientError}><Icon name="upload" size={14} />{t("runes.push")}</button>
      {/if}
      <button class="btn sm ghost" onclick={() => share(p)} title={t("runes.copyCode")}><Icon name="copy" size={14} /></button>
      {#if kind === "lib"}<button class="btn sm ghost" onclick={() => remove(p)} title={t("common.delete")}><Icon name="trash" size={14} /></button>{/if}
    </div>
  </div>
{/snippet}

<div class="page">
  <div class="page-header">
    <div>
      <h1>{t("nav.runes")}</h1>
      <p class="muted small">{t("runes.subtitle")}</p>
    </div>
    <div class="spacer"></div>
    <button class="btn" onclick={() => (importOpen = true)}><Icon name="upload" size={15} />{t("runes.importCode")}</button>
    <button class="btn" onclick={loadClient}><Icon name="refresh" size={15} />{t("common.refresh")}</button>
  </div>

  <div class="section-title">{t("runes.client")}</div>
  {#if loadingClient}
    <p class="faint">{t("common.loading")}</p>
  {:else if clientError}
    <div class="notice glass"><Icon name="info" size={16} />{t("runes.clientClosed")}</div>
  {:else}
    <div class="grid">
      {#each clientPages as p}{@render pageCard(p, "client")}{:else}<p class="faint">{t("runes.noPages")}</p>{/each}
    </div>
  {/if}

  <div class="section-title">{t("runes.library")}</div>
  <div class="grid">
    {#each library as p (p.id)}{@render pageCard(p, "lib")}{:else}<p class="faint">{t("runes.emptyLibrary")}</p>{/each}
  </div>
</div>

<Modal bind:open={importOpen} title={t("runes.importCode")} width={460}>
  <input class="input mono" placeholder="RSR1.8100.8300.…" bind:value={code} style="width:100%" />
  {#snippet footer()}
    <button class="btn" onclick={() => (importOpen = false)}>{t("common.cancel")}</button>
    <button class="btn primary" onclick={doImport} disabled={!code.trim()}>{t("configs.import")}</button>
  {/snippet}
</Modal>

<style>
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
    gap: 10px;
  }
  .rune {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 14px;
  }
  .style {
    width: 26px;
    height: 26px;
  }
  .small-s {
    width: 20px;
    height: 20px;
    opacity: 0.8;
  }
  .perks {
    display: flex;
    gap: 6px;
    align-items: center;
  }
  .perk {
    width: 30px;
    height: 30px;
    border-radius: 50%;
    background: rgba(0, 0, 0, 0.3);
  }
  .perk.key {
    width: 40px;
    height: 40px;
  }
  .perk.ph {
    display: inline-block;
  }
  .actions {
    gap: 4px;
  }
  .notice {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 12px 14px;
    border-radius: var(--radius-md);
    color: var(--text-2);
  }
  .source {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    margin-top: -2px;
  }
  .source .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
  }
</style>
