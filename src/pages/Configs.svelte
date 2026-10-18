<script lang="ts">
  // Profils de config LoL / Valorant : capture, édition des touches, application, partage, association aux comptes.
  import { open as openDialog, save as saveDialog } from "@tauri-apps/plugin-dialog";
  import type { ConfigProfile, ValDir } from "../lib/types";
  import { api } from "../lib/api";
  import { app } from "../lib/app.svelte";
  import { errorText, t } from "../lib/i18n.svelte";
  import { relativeTime } from "../lib/format";
  import { loadLayout } from "../lib/keys";
  import Avatar from "../components/Avatar.svelte";
  import GameIcon from "../components/GameIcon.svelte";
  import Icon from "../components/Icon.svelte";
  import IniTable from "../components/IniTable.svelte";
  import LolKeybinds from "../components/LolKeybinds.svelte";
  import Modal from "../components/Modal.svelte";
  import Popover from "../components/Popover.svelte";
  import ValKeybinds from "../components/ValKeybinds.svelte";

  const LOL_CATS = ["keybinds", "interface", "sound", "graphics"];
  const VAL_CATS = ["keybinds", "settings", "graphics"];
  const LOL_SECTIONS: Record<string, string[]> = {
    interface: ["HUD", "FloatingText", "LossOfControl", "Chat", "MapSkinOptions", "ItemShop", "TFT", "Highlights"],
    sound: ["Volume", "Voice"],
    graphics: ["Performance", "General"],
  };
  const VAL_KEYBINDS = "WindowsClient/BackupKeybinds.json";

  let game = $state<"lol" | "val">("lol");
  let selectedId = $state<string | null>(null);
  let draft = $state<ConfigProfile | null>(null);
  let dirty = $state(false);
  let tab = $state<"keys" | "settings">("keys");
  let valDirs = $state<ValDir[]>([]);
  let busy = $state(false);

  // Modales
  let captureOpen = $state(false);
  let capName = $state("");
  let capCats = $state<string[]>([]);
  let capDir = $state("");
  let importOpen = $state(false);
  let importCode = $state("");
  let assignOpen = $state(false);
  let assignIds = $state<string[]>([]);
  let applyOpen = $state(false);
  let applyDir = $state("");
  let shareMenu = $state(false);
  let deleteOpen = $state(false);

  loadLayout();

  const list = $derived(app.profiles.filter((p) => p.game === game));

  $effect(() => {
    if (game === "val") api.listValDirs().then((d) => (valDirs = d));
  });

  $effect(() => {
    // Sélection par défaut : premier profil du jeu.
    if (!list.find((p) => p.id === selectedId)) select(list[0]?.id ?? null);
  });

  async function select(id: string | null) {
    // Pas de perte silencieuse : les modifications en cours sont enregistrées avant de changer de profil.
    if (dirty && draft) await saveDraft();
    selectedId = id;
    dirty = false;
    tab = "keys";
    draft = id ? await api.getProfile(id) : null;
  }

  function markDirty() {
    dirty = true;
  }

  function accountName(puuid: string): string {
    const a = app.accounts.find((x) => x.puuid === puuid);
    return a ? a.label : t("configs.unknownAccount");
  }

  function openCapture() {
    capName = game === "lol" ? t("configs.defaultNameLol") : t("configs.defaultNameVal");
    const active = app.activeAccount;
    if (active) capName = `${active.label} — ${t("configs.keysWord")}`;
    capCats = game === "lol" ? ["keybinds"] : ["keybinds", "settings"];
    capDir = game === "val" ? (valDirs.find((d) => d.puuid === active?.puuid)?.name ?? valDirs[0]?.name ?? "") : "";
    captureOpen = true;
  }

  async function doCapture() {
    busy = true;
    try {
      const p = await api.captureProfile(game, capName, capCats, game === "val" ? capDir : null, app.activeAccount?.id ?? null);
      captureOpen = false;
      await app.refreshProfiles();
      await select(p.id);
      app.toast("success", p.source === "client" ? t("configs.capturedClient") : t("configs.capturedFiles"));
    } catch (e) {
      app.toast("error", errorText(e), 6000);
    } finally {
      busy = false;
    }
  }

  async function saveDraft() {
    if (!draft) return;
    try {
      const saved = await api.saveProfile($state.snapshot(draft) as ConfigProfile);
      draft = saved;
      dirty = false;
      await app.refreshProfiles();
      app.toast("success", t("configs.saved"));
    } catch (e) {
      app.toast("error", errorText(e));
    }
  }

  async function apply() {
    if (!draft) return;
    if (dirty) await saveDraft();
    if (draft.game === "val" && !applyDir) {
      applyDir = valDirs.find((d) => d.puuid === app.activeAccount?.puuid)?.name ?? valDirs[0]?.name ?? "";
      applyOpen = true;
      return;
    }
    busy = true;
    try {
      const done = await api.applyProfile(draft.id, draft.game === "val" ? applyDir : null);
      applyOpen = false;
      app.toast("success", done.includes("client") ? t("configs.appliedClient") : t("configs.appliedFiles"), 6000);
    } catch (e) {
      app.toast("error", errorText(e), 6000);
    } finally {
      busy = false;
      applyDir = "";
    }
  }

  function openAssign() {
    if (!draft) return;
    const id = draft.id;
    assignIds = app.accounts.filter((a) => (draft!.game === "lol" ? a.profiles.lol : a.profiles.val) === id).map((a) => a.id);
    assignOpen = true;
  }

  async function doAssign() {
    if (!draft) return;
    await api.assignProfile(draft.id, assignIds);
    assignOpen = false;
    app.toast("success", t("configs.assigned", { n: assignIds.length }));
    app.refreshAccounts();
  }

  async function copyCode() {
    if (!draft) return;
    shareMenu = false;
    const code = await api.exportProfileCode(draft.id);
    await navigator.clipboard.writeText(code);
    app.toast("success", t("configs.codeCopied", { n: code.length }));
  }

  async function exportFile() {
    if (!draft) return;
    shareMenu = false;
    const path = await saveDialog({ defaultPath: `${draft.name}.rsconfig`, filters: [{ name: "RiftSwitch config", extensions: ["rsconfig"] }] });
    if (!path) return;
    await api.exportProfileFile(draft.id, path);
    app.toast("success", t("configs.exported"));
  }

  async function importFromCode() {
    try {
      const p = await api.importProfileCode(importCode);
      importOpen = false;
      importCode = "";
      await app.refreshProfiles();
      game = p.game;
      await select(p.id);
      app.toast("success", t("configs.imported"));
    } catch (e) {
      app.toast("error", errorText(e));
    }
  }

  async function importFromFile() {
    const path = await openDialog({ filters: [{ name: "RiftSwitch config", extensions: ["rsconfig", "json"] }] });
    if (!path || Array.isArray(path)) return;
    try {
      const p = await api.importProfileFile(path);
      importOpen = false;
      await app.refreshProfiles();
      game = p.game;
      await select(p.id);
      app.toast("success", t("configs.imported"));
    } catch (e) {
      app.toast("error", errorText(e));
    }
  }

  async function duplicate() {
    if (!draft) return;
    const p = await api.duplicateProfile(draft.id, `${draft.name} (2)`);
    await app.refreshProfiles();
    await select(p.id);
  }

  async function remove() {
    if (!draft) return;
    await api.deleteProfile(draft.id);
    deleteOpen = false;
    dirty = false;
    await app.refreshProfiles();
    selectedId = null;
  }

  function toggleCat(c: string) {
    if (!draft) return;
    draft.categories = draft.categories.includes(c) ? draft.categories.filter((x) => x !== c) : [...draft.categories, c];
    markDirty();
  }

  const linked = $derived(
    draft ? app.accounts.filter((a) => (draft!.game === "lol" ? a.profiles.lol : a.profiles.val) === draft!.id) : [],
  );
  const settingsSections = $derived(
    draft?.game === "lol" ? Object.entries(LOL_SECTIONS).filter(([c]) => draft!.categories.includes(c)).flatMap(([, s]) => s) : [],
  );
</script>

<div class="page">
  <div class="page-header">
    <div>
      <h1>{t("nav.configs")}</h1>
      <p class="muted small">{t("configs.subtitle")}</p>
    </div>
    <div class="spacer"></div>
    <div class="seg">
      <button class:on={game === "lol"} onclick={() => (game = "lol")}><GameIcon game="lol" size={16} />League of Legends</button>
      <button class:on={game === "val"} onclick={() => (game = "val")}><GameIcon game="val" size={16} />VALORANT</button>
    </div>
    <button class="btn" onclick={() => (importOpen = true)}><Icon name="upload" size={16} />{t("configs.import")}</button>
    <button class="btn primary" onclick={openCapture}><Icon name="download" size={16} />{t("configs.capture")}</button>
  </div>

  <div class="info glass">
    <Icon name="info" size={16} />
    <span>{game === "lol" ? t("configs.howLol") : t("configs.howVal")}</span>
    {#if game === "val"}<span class="risk medium">● {t("risk.experimental")}</span>{/if}
  </div>

  <div class="layout">
    <aside class="list">
      {#each list as p (p.id)}
        {@const users = app.accounts.filter((a) => (p.game === "lol" ? a.profiles.lol : a.profiles.val) === p.id)}
        <button class="item glass" class:sel={p.id === selectedId} onclick={() => select(p.id)}>
          <div class="row">
            <strong class="ellipsis grow">{p.name}</strong>
            <span class="faint small">{relativeTime(p.updatedAt)}</span>
          </div>
          <div class="row" style="flex-wrap:wrap;gap:4px">
            {#each p.categories as c}<span class="chip">{t(`configs.cat_${c}`)}</span>{/each}
          </div>
          {#if users.length}
            <div class="avatars">
              {#each users.slice(0, 6) as u}<Avatar icon={u.icon} color={u.color} size={20} label={u.label} />{/each}
              {#if users.length > 6}<span class="faint small">+{users.length - 6}</span>{/if}
            </div>
          {/if}
        </button>
      {:else}
        <div class="empty glass">
          <Icon name="keyboard" size={30} />
          <p>{t("configs.empty")}</p>
          <button class="btn primary sm" onclick={openCapture}>{t("configs.capture")}</button>
        </div>
      {/each}
    </aside>

    {#if draft}
      <section class="detail glass">
        <div class="dhead">
          <input class="input name" bind:value={draft.name} oninput={markDirty} />
          {#if dirty}<button class="btn primary sm" onclick={saveDraft}><Icon name="check" size={15} />{t("common.save")}</button>{/if}
          <button class="btn sm primary" onclick={apply} disabled={busy}><Icon name="play" size={13} />{t("configs.applyNow")}</button>
          <button class="btn sm" onclick={openAssign}><Icon name="users" size={15} />{t("configs.assign")}</button>
          <Popover bind:open={shareMenu} width={240}>
            {#snippet trigger()}
              <button class="btn sm icon" onclick={() => (shareMenu = !shareMenu)} title={t("configs.share")}><Icon name="more" /></button>
            {/snippet}
            <button class="menu-item" onclick={copyCode}><Icon name="copy" size={16} />{t("configs.copyCode")}</button>
            <button class="menu-item" onclick={exportFile}><Icon name="download" size={16} />{t("configs.exportFile")}</button>
            <button class="menu-item" onclick={() => ((shareMenu = false), duplicate())}><Icon name="copy" size={16} />{t("configs.duplicate")}</button>
            <div class="divider"></div>
            <button class="menu-item danger" onclick={() => ((shareMenu = false), (deleteOpen = true))}><Icon name="trash" size={16} />{t("common.delete")}</button>
          </Popover>
        </div>

        <div class="meta row small muted">
          <span>{t(`configs.source_${draft.source}`)}</span>
          {#if linked.length}
            <span>·</span><span>{t("configs.usedBy", { n: linked.length })}</span>
            <span class="avatars">{#each linked.slice(0, 5) as u}<Avatar icon={u.icon} color={u.color} size={18} label={u.label} />{/each}</span>
          {/if}
        </div>

        <div class="cats">
          <span class="label">{t("configs.appliesTo")}</span>
          {#each draft.game === "lol" ? LOL_CATS : VAL_CATS as c}
            <button class="chip" class:accent={draft.categories.includes(c)} style="cursor:pointer" onclick={() => toggleCat(c)}>
              {#if draft.categories.includes(c)}<Icon name="check" size={12} />{/if}{t(`configs.cat_${c}`)}
            </button>
          {/each}
        </div>

        <div class="tabs">
          <button class:on={tab === "keys"} onclick={() => (tab = "keys")}><Icon name="keyboard" size={15} />{t("configs.tabKeys")}</button>
          <button class:on={tab === "settings"} onclick={() => (tab = "settings")}><Icon name="settings" size={15} />{t("configs.tabSettings")}</button>
        </div>

        <div class="tab-body">
          {#if tab === "keys"}
            {#if draft.lol}
              <LolKeybinds bind:input={draft.lol.input} onchange={markDirty} />
            {:else if draft.val}
              <ValKeybinds bind:json={draft.val.files[VAL_KEYBINDS]} onchange={markDirty} />
            {/if}
          {:else if draft.lol}
            {#if settingsSections.length}
              <IniTable bind:doc={draft.lol.game} sections={settingsSections} onchange={markDirty} />
            {:else}
              <p class="faint small">{t("configs.noCategory")}</p>
            {/if}
          {:else if draft.val}
            {#each Object.entries(draft.val.files).filter(([k]) => k !== VAL_KEYBINDS) as [file, text]}
              <div class="section-title" style="margin-top:4px">{file}</div>
              <pre class="raw mono small">{text.slice(0, 4000)}</pre>
            {/each}
          {/if}
        </div>
      </section>
    {:else if list.length}
      <section class="detail glass empty"><p>{t("configs.pick")}</p></section>
    {/if}
  </div>
</div>

<Modal bind:open={captureOpen} title={t("configs.captureTitle")} width={500}>
  <div class="col">
    <p class="muted small">{game === "lol" ? t("configs.captureLolHint") : t("configs.captureValHint")}</p>
    <div class="field">
      <label for="cn">{t("configs.name")}</label>
      <input id="cn" class="input" bind:value={capName} />
    </div>
    {#if game === "val"}
      <div class="field">
        <label for="cd">{t("configs.valAccountDir")}</label>
        <select id="cd" class="input" bind:value={capDir}>
          {#each valDirs as d}<option value={d.name}>{accountName(d.puuid)} · {d.shard.toUpperCase()} · {relativeTime(d.modified)}</option>{/each}
        </select>
        {#if !valDirs.length}<p class="small" style="color:var(--warning)">{t("configs.noValDirs")}</p>{/if}
      </div>
    {/if}
    <span class="label">{t("configs.categories")}</span>
    <div class="row" style="flex-wrap:wrap;gap:6px">
      {#each game === "lol" ? LOL_CATS : VAL_CATS as c}
        <button
          class="chip"
          class:accent={capCats.includes(c)}
          style="cursor:pointer"
          onclick={() => (capCats = capCats.includes(c) ? capCats.filter((x) => x !== c) : [...capCats, c])}>{t(`configs.cat_${c}`)}</button
        >
      {/each}
    </div>
  </div>
  {#snippet footer()}
    <button class="btn" onclick={() => (captureOpen = false)}>{t("common.cancel")}</button>
    <button class="btn primary" onclick={doCapture} disabled={busy || !capCats.length || (game === "val" && !capDir)}>{t("configs.capture")}</button>
  {/snippet}
</Modal>

<Modal bind:open={importOpen} title={t("configs.importTitle")} width={520}>
  <div class="col">
    <div class="field">
      <label for="ic">{t("configs.pasteCode")}</label>
      <textarea id="ic" class="input mono small" placeholder="RSC1.…" bind:value={importCode}></textarea>
    </div>
    <div class="row">
      <button class="btn" onclick={importFromFile}><Icon name="upload" size={15} />{t("configs.fromFile")}</button>
      <span class="spacer"></span>
      <button class="btn primary" onclick={importFromCode} disabled={!importCode.trim()}>{t("configs.import")}</button>
    </div>
  </div>
</Modal>

<Modal bind:open={assignOpen} title={t("configs.assignTitle")} width={500}>
  <p class="muted small">{t("configs.assignHint")}</p>
  <div class="row" style="margin-bottom:8px">
    <button class="btn sm" onclick={() => (assignIds = app.accounts.map((a) => a.id))}>{t("configs.selectAll")}</button>
    <button class="btn sm ghost" onclick={() => (assignIds = [])}>{t("configs.selectNone")}</button>
  </div>
  <div class="assign">
    {#each app.accounts as a (a.id)}
      <label class="arow">
        <input
          type="checkbox"
          checked={assignIds.includes(a.id)}
          onchange={(e) => (assignIds = e.currentTarget.checked ? [...assignIds, a.id] : assignIds.filter((x) => x !== a.id))}
        />
        <Avatar icon={a.icon} color={a.color} size={28} label={a.label} />
        <span class="grow">{a.label}</span>
        <span class="faint small">{a.region}</span>
      </label>
    {/each}
  </div>
  {#snippet footer()}
    <button class="btn" onclick={() => (assignOpen = false)}>{t("common.cancel")}</button>
    <button class="btn primary" onclick={doAssign}>{t("common.save")}</button>
  {/snippet}
</Modal>

<Modal bind:open={applyOpen} title={t("configs.applyVal")} width={460}>
  <div class="field">
    <label for="ad">{t("configs.valAccountDir")}</label>
    <select id="ad" class="input" bind:value={applyDir}>
      {#each valDirs as d}<option value={d.name}>{accountName(d.puuid)} · {d.shard.toUpperCase()}</option>{/each}
    </select>
  </div>
  <p class="faint small">{t("configs.valWarn")}</p>
  {#snippet footer()}
    <button class="btn" onclick={() => ((applyOpen = false), (applyDir = ""))}>{t("common.cancel")}</button>
    <button class="btn primary" onclick={apply} disabled={!applyDir}>{t("configs.applyNow")}</button>
  {/snippet}
</Modal>

<Modal bind:open={deleteOpen} title={t("configs.deleteTitle")} width={420}>
  <p>{t("configs.deleteText", { name: draft?.name ?? "" })}</p>
  {#snippet footer()}
    <button class="btn" onclick={() => (deleteOpen = false)}>{t("common.cancel")}</button>
    <button class="btn danger solid" onclick={remove}>{t("common.delete")}</button>
  {/snippet}
</Modal>

<style>
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
    gap: 7px;
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
    box-shadow: 0 1px 4px rgba(0, 0, 0, 0.2);
  }
  .info {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 14px;
    border-radius: var(--radius-md);
    font-size: 12.5px;
    color: var(--text-2);
    margin-bottom: 16px;
  }
  .info :global(svg) {
    color: var(--info);
  }
  .layout {
    display: grid;
    grid-template-columns: 290px 1fr;
    gap: 16px;
    align-items: start;
  }
  .list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .item {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 12px 14px;
    text-align: start;
    cursor: pointer;
    color: var(--text);
    border-radius: var(--radius-md);
  }
  .item.sel {
    border-color: var(--accent);
    background: var(--accent-soft);
  }
  .avatars {
    display: flex;
    align-items: center;
    gap: 2px;
  }
  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
    padding: 30px 16px;
    text-align: center;
    color: var(--text-2);
  }
  .detail {
    padding: 18px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    min-width: 0;
  }
  .dhead {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .name {
    flex: 1;
    font-size: 17px;
    font-weight: 650;
  }
  .cats {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-wrap: wrap;
  }
  .tabs {
    display: flex;
    gap: 2px;
    border-bottom: 1px solid var(--border);
  }
  .tabs button {
    display: flex;
    align-items: center;
    gap: 6px;
    border: none;
    background: none;
    color: var(--text-2);
    padding: 9px 12px;
    cursor: pointer;
    font-weight: 600;
    border-bottom: 2px solid transparent;
    margin-bottom: -1px;
  }
  .tabs button.on {
    color: var(--text);
    border-bottom-color: var(--accent);
  }
  .raw {
    max-height: 260px;
    overflow: auto;
    padding: 10px;
    border-radius: var(--radius-sm);
    background: var(--surface);
    border: 1px solid var(--border);
    white-space: pre-wrap;
    user-select: text;
  }
  .assign {
    display: flex;
    flex-direction: column;
    gap: 2px;
    max-height: 340px;
    overflow-y: auto;
  }
  .arow {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 6px 8px;
    border-radius: var(--radius-sm);
    cursor: pointer;
  }
  .arow:hover {
    background: var(--surface-hover);
  }
</style>
