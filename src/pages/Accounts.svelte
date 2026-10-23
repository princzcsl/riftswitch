<script lang="ts">
  import type { Account } from "../lib/types";
  import { api } from "../lib/api";
  import { app } from "../lib/app.svelte";
  import { errorText, t } from "../lib/i18n.svelte";
  import { bestRank, rankScore } from "../lib/format";
  import AccountCard from "../components/AccountCard.svelte";
  import AccountAdd from "../components/AccountAdd.svelte";
  import ContextMenu from "../components/ContextMenu.svelte";
  import AccountEditor from "../components/AccountEditor.svelte";
  import FolderManager from "../components/FolderManager.svelte";
  import HeroCard from "../components/HeroCard.svelte";
  import Icon from "../components/Icon.svelte";
  import Modal from "../components/Modal.svelte";

  let q = $state("");
  let folder = $state<string>("all");
  let sort = $state<"manual" | "recent" | "name" | "rank">("recent");
  let addOpen = $state(false);
  let ctx = $state<{ open: boolean; x: number; y: number; kind: "folder" | "account"; id: string }>({ open: false, x: 0, y: 0, kind: "account", id: "" });
  function openCtx(e: MouseEvent, kind: "folder" | "account", id: string) {
    ctx = { open: true, x: e.clientX, y: e.clientY, kind, id };
  }
  async function ctxAction(run: () => Promise<unknown>) {
    ctx.open = false;
    try {
      await run();
      await app.refreshAccounts();
    } catch (e) {
      app.toast("error", errorText(e));
    }
  }
  let editorOpen = $state(false);
  let editing = $state<Account | null>(null);
  let foldersOpen = $state(false);
  let confirmDelete = $state<Account | null>(null);
  let confirmOpen = $state(false);

  try {
    sort = (localStorage.getItem("rs.sort") as typeof sort) || "recent";
  } catch {
    /* ignoré */
  }

  const streamer = $derived(app.settings?.streamerMode ?? false);

  const filtered = $derived.by(() => {
    let list = app.accounts.slice();
    if (folder === "fav") list = list.filter((a) => a.favorite);
    else if (folder !== "all") list = list.filter((a) => a.folderId === folder);
    const s = q.trim().toLowerCase();
    if (s)
      list = list.filter((a) =>
        [a.label, streamer ? "" : a.riotId, a.region, ...a.tags, ...a.championPool, ...a.agentPool].some((x) => x?.toLowerCase().includes(s)),
      );
    if (sort === "recent") list.sort((a, b) => (b.lastUsedAt ?? b.createdAt) - (a.lastUsedAt ?? a.createdAt));
    else if (sort === "name") list.sort((a, b) => a.label.localeCompare(b.label));
    else if (sort === "rank")
      list.sort((a, b) => {
        const ra = bestRank(a);
        const rb = bestRank(b);
        return rankScore(rb?.rank, rb?.game === "val") - rankScore(ra?.rank, ra?.game === "val");
      });
    // Favoris d'abord, à tri égal.
    return list.sort((a, b) => Number(b.favorite) - Number(a.favorite));
  });

  const hero = $derived.by(() => {
    if (q || folder !== "all" || !app.accounts.length) return null;
    return app.activeAccount ?? app.accounts.slice().sort((a, b) => (b.lastUsedAt ?? 0) - (a.lastUsedAt ?? 0))[0];
  });

  function edit(a: Account | null) {
    editing = a;
    editorOpen = true;
  }

  async function doDelete() {
    if (!confirmDelete) return;
    await api.deleteAccount(confirmDelete.id);
    confirmOpen = false;
    app.toast("info", t("accounts.deleted"));
    app.refreshAccounts();
  }

  function setSort(v: typeof sort) {
    sort = v;
    try {
      localStorage.setItem("rs.sort", v);
    } catch {
      /* ignoré */
    }
  }
</script>

<div class="page">
  <div class="page-header">
    <div>
      <h1>{t("nav.accounts")}</h1>
      <p class="muted small">{t("accounts.count", { n: app.accounts.length })}</p>
    </div>
    <div class="spacer"></div>
    <div class="search">
      <Icon name="search" size={16} />
      <input placeholder={t("accounts.searchPh")} bind:value={q} />
    </div>
    <select class="input sort" value={sort} onchange={(e) => setSort(e.currentTarget.value as typeof sort)} aria-label={t("accounts.sort")}>
      <option value="recent">{t("accounts.sortRecent")}</option>
      <option value="name">{t("accounts.sortName")}</option>
      <option value="rank">{t("accounts.sortRank")}</option>
    </select>
    <button class="btn primary" onclick={() => (addOpen = true)}><Icon name="plus" size={16} />{t("accounts.add")}</button>
  </div>

  {#if app.accounts.length}
    <div class="folders">
      <button class="fchip" class:on={folder === "all"} onclick={() => (folder = "all")}>{t("accounts.allFolders")}</button>
      <button class="fchip" class:on={folder === "fav"} onclick={() => (folder = "fav")}><Icon name="starFill" size={13} />{t("accounts.favorites")}</button>
      {#each app.folders as f (f.id)}
        <button
          class="fchip"
          class:on={folder === f.id}
          onclick={() => (folder = f.id)}
          oncontextmenu={(e) => (e.preventDefault(), openCtx(e, "folder", f.id))}
        >
          <span class="dot" style="color:{f.color || 'var(--text-3)'}"></span>{f.name}
          <span class="faint">{app.accounts.filter((a) => a.folderId === f.id).length}</span>
        </button>
      {/each}
      <button class="fchip ghost" onclick={() => (foldersOpen = true)}><Icon name="folder" size={14} />{t("folders.manage")}</button>
    </div>

    {#if hero}<HeroCard account={hero} />{/if}

    <div class="grid">
      {#each filtered as account (account.id)}
        <AccountCard
          {account}
          showFolder={folder === "all" || folder === "fav"}
          onedit={() => edit(account)}
          onfavorite={() => api.setFavorite(account.id, !account.favorite).then(() => app.refreshAccounts())}
          ondelete={() => ((confirmDelete = account), (confirmOpen = true))}
          oncontext={(e) => openCtx(e, "account", account.id)}
        />
      {:else}
        <p class="empty">{t("accounts.noMatch")}</p>
      {/each}
    </div>
  {:else}
    <div class="welcome glass">
      <img src="/logo.svg" alt="" width="88" height="88" />
      <h2>{t("accounts.emptyTitle")}</h2>
      <p class="muted">{t("accounts.emptyText")}</p>
      <button class="btn primary lg" onclick={() => (addOpen = true)}><Icon name="plus" size={18} />{t("accounts.add")}</button>
    </div>
  {/if}
</div>

<AccountAdd bind:open={addOpen} />
<AccountEditor bind:open={editorOpen} account={editing} />
<FolderManager bind:open={foldersOpen} />

<ContextMenu bind:open={ctx.open} x={ctx.x} y={ctx.y}>
  {#if ctx.kind === "folder"}
    <button class="menu-item" onclick={() => ((ctx.open = false), (foldersOpen = true))}><Icon name="edit" size={16} />{t("folders.manage")}</button>
    <button class="menu-item danger" onclick={() => ctxAction(() => api.deleteFolder(ctx.id).then(() => folder === ctx.id && (folder = "all")))}><Icon name="trash" size={16} />{t("common.delete")}</button>
  {:else}
    {@const acc = app.accounts.find((a) => a.id === ctx.id)}
    <button class="menu-item" onclick={() => ((ctx.open = false), acc && edit(acc))}><Icon name="edit" size={16} />{t("common.edit")}</button>
    <button class="menu-item" onclick={() => acc && ctxAction(() => api.setFavorite(acc.id, !acc.favorite))}><Icon name={acc?.favorite ? "starFill" : "star"} size={16} />{t("accounts.favorite")}</button>
    <button class="menu-item danger" onclick={() => ((ctx.open = false), acc && ((confirmDelete = acc), (confirmOpen = true)))}><Icon name="trash" size={16} />{t("common.delete")}</button>
  {/if}
</ContextMenu>

<Modal bind:open={confirmOpen} title={t("accounts.deleteTitle")} width={420}>
  <p>{t("accounts.deleteText", { name: confirmDelete?.label ?? "" })}</p>
  {#snippet footer()}
    <button class="btn" onclick={() => (confirmOpen = false)}>{t("common.cancel")}</button>
    <button class="btn danger solid" onclick={doDelete}>{t("common.delete")}</button>
  {/snippet}
</Modal>

<style>
  .search {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 36px;
    padding: 0 12px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--text-3);
    width: 240px;
  }
  .search input {
    border: none;
    outline: none;
    background: transparent;
    flex: 1;
    min-width: 0;
    color: var(--text);
    user-select: text;
  }
  .sort {
    height: 36px;
    width: auto;
  }
  .folders {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-bottom: 16px;
  }
  .fchip {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 30px;
    padding: 0 12px;
    border-radius: var(--radius-pill);
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--text-2);
    cursor: pointer;
    font-weight: 600;
    font-size: 12.5px;
  }
  .fchip:hover {
    color: var(--text);
  }
  .fchip.on {
    background: var(--accent-soft);
    color: var(--text);
    border-color: color-mix(in srgb, var(--accent) 50%, transparent);
  }
  .fchip.ghost {
    border-style: dashed;
    background: transparent;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 14px;
    margin-top: 14px;
  }
  .grid .empty {
    grid-column: 1 / -1;
  }
  .welcome {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: 10px;
    padding: 40px 30px;
    margin-top: 20px;
  }
  .welcome img {
    filter: drop-shadow(0 10px 30px var(--accent-soft));
  }
  .welcome .btn {
    margin-top: 16px;
  }
</style>
