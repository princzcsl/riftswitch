<script lang="ts">
  // Profils de config LoL / Valorant : capture, édition des touches, application, partage, association aux comptes.
  import { open as openDialog, save as saveDialog } from "@tauri-apps/plugin-dialog";
  import type { Account, ConfigProfile, IniDoc, ValDir } from "../lib/types";
  import { api } from "../lib/api";
  import { app } from "../lib/app.svelte";
  import { errorText, t } from "../lib/i18n.svelte";
  import { relativeTime } from "../lib/format";
  import { iniText, lolCat, parseIni, valCat } from "../lib/ini";
  import { LOL_SCHEMA, VAL_SCHEMA } from "../lib/settingsSchema";
  import { loadLayout } from "../lib/keys";
  import Avatar from "../components/Avatar.svelte";
  import GameIcon from "../components/GameIcon.svelte";
  import Icon from "../components/Icon.svelte";
  import CrosshairProfiles from "../components/CrosshairProfiles.svelte";
  import LolKeybinds from "../components/LolKeybinds.svelte";
  import Modal from "../components/Modal.svelte";
  import Popover from "../components/Popover.svelte";
  import SettingsPanel from "../components/SettingsPanel.svelte";
  import ValKeybinds from "../components/ValKeybinds.svelte";

  // Catégories : elles filtrent l'affichage et ce qui est appliqué. Tout est toujours capturé.
  const LOL_CATS = ["keybinds", "interface", "sound", "graphics"];
  const VAL_CATS = ["keybinds", "interface", "crosshair", "sound", "graphics"];
  const catsOf = (g: string) => (g === "lol" ? LOL_CATS : VAL_CATS);
  const CAT_ICONS: Record<string, string> = { keybinds: "keyboard", interface: "image", crosshair: "target", sound: "signal", graphics: "sparkle" };
  const VAL_KEYBINDS = "WindowsClient/BackupKeybinds.json";
  const VAL_SETTINGS = "Windows/RiotUserSettings.ini";
  const VAL_GRAPHICS = "WindowsClient/GameUserSettings.ini";

  let game = $state<"lol" | "val">("lol");
  let selectedId = $state<string | null>(null);
  let draft = $state<ConfigProfile | null>(null);
  let dirty = $state(false);
  let tab = $state("keybinds");
  let valDocs = $state<Record<string, IniDoc>>({});
  let valDirs = $state<ValDir[]>([]);
  let busy = $state(false);

  // Modales
  let captureOpen = $state(false);
  let capName = $state("");
  let capNameEdited = $state(false);
  let capCats = $state<string[]>([]);
  let capDir = $state("");
  let importOpen = $state(false);
  let importCode = $state("");
  let applyOpen = $state(false);
  let applyIds = $state<string[]>([]);
  let shareMenu = $state(false);
  let moreMenu = $state(false);
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

  /** Anciens profils VALORANT : « settings » couvrait interface, viseur et son. */
  function normalize(p: ConfigProfile): ConfigProfile {
    if (p.categories.includes("settings")) p.categories = [...new Set(p.categories.flatMap((c) => (c === "settings" ? ["interface", "crosshair", "sound"] : [c])))];
    if (!p.sourceAccount && p.notes.startsWith("account:")) p.sourceAccount = p.notes.slice(8);
    return p;
  }

  async function select(id: string | null) {
    // Pas de perte silencieuse : les modifications en cours sont enregistrées avant de changer de profil.
    if (dirty && draft) await saveDraft();
    selectedId = id;
    dirty = false;
    draft = id ? normalize(await api.getProfile(id)) : null;
    valDocs = {};
    for (const f of [VAL_SETTINGS, VAL_GRAPHICS]) if (draft?.val?.files[f]) valDocs[f] = parseIni(draft.val.files[f]);
    tab = catsOf(draft?.game ?? game).find((c) => draft?.categories.includes(c)) ?? "keybinds";
  }

  function markDirty() {
    dirty = true;
  }

  function valChanged(file: string) {
    if (!draft?.val) return;
    draft.val.files[file] = iniText($state.snapshot(valDocs[file]));
    markDirty();
  }

  const riotId = (a: Account) => (a.gameName ? `${a.gameName}#${a.tagLine}` : a.label);
  const nameOf = (a: Account | undefined) => (a ? a.label || a.gameName || t("configs.unknownAccount") : t("configs.unknownAccount"));
  const byPuuid = (puuid: string) => app.accounts.find((x) => x.puuid === puuid);
  const dirLabel = (d: ValDir) => {
    const a = byPuuid(d.puuid);
    const who = a ? (a.label && a.gameName ? `${a.label} (${riotId(a)})` : riotId(a) || nameOf(a)) : t("configs.unknownAccount");
    return `${who} · ${d.shard.toUpperCase()} · ${relativeTime(d.modified)}`;
  };

  /** Compte d'origine d'une capture : le dossier VALORANT choisi, sinon le compte connecté ou le dernier lancé sur ce jeu. */
  function captureSource(): Account | undefined {
    if (game === "val") return byPuuid(valDirs.find((d) => d.name === capDir)?.puuid ?? "") ?? undefined;
    const games = ["lol", "tft"];
    return (
      app.activeAccount ??
      [...app.accounts].filter((a) => games.includes(a.lastGame ?? "")).sort((a, b) => (b.lastUsedAt ?? 0) - (a.lastUsedAt ?? 0))[0]
    );
  }

  function defaultName(): string {
    const a = captureSource();
    return a?.gameName || a?.label || (game === "lol" ? t("configs.defaultNameLol") : t("configs.defaultNameVal"));
  }

  function openCapture() {
    const active = app.activeAccount;
    capCats = [...catsOf(game)];
    capDir = game === "val" ? (valDirs.find((d) => d.puuid === active?.puuid)?.name ?? valDirs[0]?.name ?? "") : "";
    capNameEdited = false;
    capName = defaultName();
    captureOpen = true;
  }

  function onCapDir() {
    if (!capNameEdited) capName = defaultName();
  }

  async function doCapture() {
    busy = true;
    try {
      const p = await api.captureProfile(game, capName, capCats, game === "val" ? capDir : null, captureSource()?.id ?? null);
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
      if (saved) draft = normalize(saved);
      dirty = false;
      await app.refreshProfiles();
      app.toast("success", t("configs.saved"));
    } catch (e) {
      app.toast("error", errorText(e));
    }
  }

  const dirOf = (a: Account) => valDirs.find((d) => d.puuid === a.puuid)?.name;

  function openApply() {
    if (!draft) return;
    const id = draft.id;
    applyIds = app.accounts.filter((a) => (draft!.game === "lol" ? a.profiles.lol : a.profiles.val) === id).map((a) => a.id);
    if (!applyIds.length && app.activeAccount) applyIds = [app.activeAccount.id];
    applyOpen = true;
  }

  /** Lie le profil aux comptes cochés (réappliqué à chaque lancement) et l'applique tout de suite quand c'est possible. */
  async function doApply() {
    if (!draft) return;
    if (dirty) await saveDraft();
    busy = true;
    try {
      await api.assignProfile(draft.id, applyIds);
      let now = 0;
      if (draft.game === "lol") {
        const active = app.activeAccount;
        if (active && applyIds.includes(active.id) && app.status.procs.lolClient) {
          await api.applyProfile(draft.id, null);
          now = 1;
        }
      } else {
        for (const a of app.accounts.filter((x) => applyIds.includes(x.id))) {
          const dir = dirOf(a);
          if (dir) (await api.applyProfile(draft.id, dir), now++);
        }
      }
      applyOpen = false;
      app.toast("success", now ? t("configs.appliedN", { n: now, total: applyIds.length }) : t("configs.assigned", { n: applyIds.length }), 6000);
      app.refreshAccounts();
      app.refreshProfiles();
    } catch (e) {
      app.toast("error", errorText(e), 6000);
    } finally {
      busy = false;
    }
  }

  async function copyCode() {
    if (!draft) return;
    shareMenu = false;
    if (dirty) await saveDraft();
    const code = await api.exportProfileCode(draft.id);
    await navigator.clipboard.writeText(code);
    app.toast("success", t("configs.codeCopied"));
  }

  async function exportFile() {
    if (!draft) return;
    shareMenu = false;
    if (dirty) await saveDraft();
    const path = await saveDialog({ defaultPath: `${draft.name}.rsconfig`, filters: [{ name: "RiftSwitch config", extensions: ["rsconfig"] }] });
    if (!path) return;
    await api.exportProfileFile(draft.id, path);
    app.toast("success", t("configs.exported"));
  }

  async function imported(p: ConfigProfile) {
    importOpen = false;
    importCode = "";
    await app.refreshProfiles();
    game = p.game;
    await select(p.id);
    app.toast("success", t("configs.imported"));
  }

  async function importFromCode() {
    try {
      await imported(await api.importProfileCode(importCode));
    } catch (e) {
      app.toast("error", errorText(e));
    }
  }

  async function importFromFile() {
    const path = await openDialog({ filters: [{ name: "RiftSwitch config", extensions: ["rsconfig", "json"] }] });
    if (!path || Array.isArray(path)) return;
    try {
      await imported(await api.importProfileFile(path));
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
    draft.categories = draft.categories.includes(c) ? draft.categories.filter((x) => x !== c) : catsOf(draft.game).filter((x) => x === c || draft!.categories.includes(x));
    if (!draft.categories.includes(tab)) tab = draft.categories[0] ?? "keybinds";
    else if (draft.categories.length === 1) tab = c;
    markDirty();
  }

  const linked = $derived(
    draft ? app.accounts.filter((a) => (draft!.game === "lol" ? a.profiles.lol : a.profiles.val) === draft!.id) : [],
  );
  const tabs = $derived(draft ? catsOf(draft.game).filter((c) => draft!.categories.includes(c)) : []);
  const source = $derived(draft?.sourceAccount ? app.accounts.find((a) => a.id === draft!.sourceAccount) : undefined);
</script>

<div class="page">
  <div class="page-header">
    <div>
      <h1>{t("nav.configs")}</h1>
      <p class="muted small">{t("configs.subtitle")}</p>
    </div>
    <div class="spacer"></div>
    <button class="btn" onclick={() => (importOpen = true)}><Icon name="upload" size={16} />{t("configs.import")}</button>
    <button class="btn primary" onclick={openCapture}><Icon name="download" size={16} />{t("configs.capture")}</button>
  </div>

  <div class="games">
    {#each [["lol", "League of Legends"], ["val", "VALORANT"]] as [g, label] (g)}
      <button class="game glass" class:on={game === g} onclick={() => (game = g as "lol" | "val")}>
        <GameIcon game={g as "lol" | "val"} size={22} />
        <span>{label}</span>
        <span class="count">{app.profiles.filter((p) => p.game === g).length}</span>
      </button>
    {/each}
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
            {#if p.source === "backup"}<span class="chip warning">{t("configs.backupChip")}</span>{/if}
            {#each normalize({ ...p }).categories as c}<span class="chip">{t(`configs.cat_${c}`)}</span>{/each}
          </div>
          {#if users.length}
            <div class="avatars">
              {#each users.slice(0, 6) as u}<Avatar icon={u.icon} autoIconId={u.profileIconId} color={u.color} size={20} label={u.label} />{/each}
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
          <button class="btn sm primary" onclick={openApply} disabled={busy}><Icon name="play" size={13} />{t("configs.applyBtn")}</button>
          <Popover bind:open={shareMenu} width={250}>
            {#snippet trigger()}
              <button class="btn sm" onclick={() => (shareMenu = !shareMenu)}><Icon name="link" size={14} />{t("configs.shareBtn")}</button>
            {/snippet}
            <button class="menu-item" onclick={copyCode}><Icon name="copy" size={16} />{t("configs.copyCode")}</button>
            <button class="menu-item" onclick={exportFile}><Icon name="download" size={16} />{t("configs.exportFile")}</button>
          </Popover>
          <Popover bind:open={moreMenu} width={200}>
            {#snippet trigger()}
              <button class="btn sm icon" onclick={() => (moreMenu = !moreMenu)} title={t("common.more")}><Icon name="more" /></button>
            {/snippet}
            <button class="menu-item" onclick={() => ((moreMenu = false), duplicate())}><Icon name="copy" size={16} />{t("configs.duplicate")}</button>
            <div class="divider"></div>
            <button class="menu-item danger" onclick={() => ((moreMenu = false), (deleteOpen = true))}><Icon name="trash" size={16} />{t("common.delete")}</button>
          </Popover>
        </div>

        <div class="meta row small muted">
          {#if source || draft.sourceName}
            <span class="src">
              <Avatar icon={source?.icon} autoIconId={source?.profileIconId ?? draft.sourceIcon} color={source?.color} size={22} label={source?.label || draft.sourceName} />
              <strong>{source ? riotId(source) : draft.sourceName}</strong>
            </span>
          {:else}
            <span>{t(`configs.source_${draft.source}`)}</span>
          {/if}
          {#if draft.source === "backup"}<span class="chip warning">{t("configs.backupChip")}</span>{/if}
          {#if linked.length}
            <span>·</span><span>{t("configs.usedBy", { n: linked.length })}</span>
            <span class="avatars">{#each linked.slice(0, 5) as u}<Avatar icon={u.icon} autoIconId={u.profileIconId} color={u.color} size={18} label={u.label} />{/each}</span>
          {/if}
        </div>

        <div class="cats">
          <span class="label">{t("configs.appliesTo")}</span>
          {#each catsOf(draft.game) as c}
            <button class="chip" class:accent={draft.categories.includes(c)} style="cursor:pointer" onclick={() => toggleCat(c)}>
              {#if draft.categories.includes(c)}<Icon name="check" size={12} />{/if}{t(`configs.cat_${c}`)}
            </button>
          {/each}
        </div>

        {#if tabs.length}
          <div class="tabs">
            {#each tabs as c (c)}
              <button class:on={tab === c} onclick={() => (tab = c)}><Icon name={CAT_ICONS[c]} size={15} />{t(`configs.cat_${c}`)}</button>
            {/each}
          </div>

          <div class="tab-body">
            {#if tab === "keybinds"}
              {#if draft.lol}
                <LolKeybinds bind:input={draft.lol.input} onchange={markDirty} />
              {:else if draft.val?.files[VAL_KEYBINDS]}
                <ValKeybinds bind:json={draft.val.files[VAL_KEYBINDS]} onchange={markDirty} />
              {:else}
                <p class="faint small">{t("configs.notCaptured")}</p>
              {/if}
            {:else if draft.lol}
              <SettingsPanel bind:doc={draft.lol.game} groups={LOL_SCHEMA[tab] ?? []} advanced={(s, k) => lolCat(s, k) === tab} onchange={markDirty} />
            {:else if draft.val}
              {@const file = tab === "graphics" ? VAL_GRAPHICS : VAL_SETTINGS}
              {#if !valDocs[file]}
                <p class="faint small">{t("configs.notCaptured")}</p>
              {:else if tab === "crosshair"}
                <CrosshairProfiles bind:doc={valDocs[file]} onchange={() => valChanged(file)} />
              {:else}
                <SettingsPanel
                  bind:doc={valDocs[file]}
                  groups={VAL_SCHEMA[tab] ?? []}
                  advanced={(_, k) => tab === "graphics" || valCat(k) === tab}
                  onchange={() => valChanged(file)}
                />
              {/if}
            {/if}
          </div>
        {:else}
          <p class="faint small">{t("configs.noCategory")}</p>
        {/if}
      </section>
    {:else if list.length}
      <section class="detail glass empty"><p>{t("configs.pick")}</p></section>
    {/if}
  </div>
</div>

<Modal bind:open={captureOpen} title={t("configs.captureTitle")} width={500}>
  <div class="col">
    <p class="muted small">{game === "lol" ? t("configs.captureLolHint") : t("configs.captureValHint")}</p>
    {#if game === "val"}
      <div class="field">
        <label for="cd">{t("configs.valAccountDir")}</label>
        <select id="cd" class="input" bind:value={capDir} onchange={onCapDir}>
          {#each valDirs as d}<option value={d.name}>{dirLabel(d)}</option>{/each}
        </select>
        {#if !valDirs.length}<p class="small" style="color:var(--warning)">{t("configs.noValDirs")}</p>{/if}
      </div>
    {/if}
    <div class="field">
      <label for="cn">{t("configs.name")}</label>
      <input id="cn" class="input" bind:value={capName} oninput={() => (capNameEdited = true)} />
    </div>
    <span class="label">{t("configs.categories")}</span>
    <div class="row" style="flex-wrap:wrap;gap:6px">
      {#each catsOf(game) as c}
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

<Modal bind:open={applyOpen} title={t("configs.applyTitle")} width={520}>
  <p class="muted small">{draft?.game === "val" ? t("configs.applyHintVal") : t("configs.applyHintLol")}</p>
  <div class="row" style="margin-bottom:8px">
    <button class="btn sm" onclick={() => (applyIds = app.accounts.map((a) => a.id))}>{t("configs.selectAll")}</button>
    <button class="btn sm ghost" onclick={() => (applyIds = [])}>{t("configs.selectNone")}</button>
  </div>
  <div class="assign">
    {#each app.accounts as a (a.id)}
      <label class="arow">
        <input
          type="checkbox"
          checked={applyIds.includes(a.id)}
          onchange={(e) => (applyIds = e.currentTarget.checked ? [...applyIds, a.id] : applyIds.filter((x) => x !== a.id))}
        />
        <Avatar icon={a.icon} autoIconId={a.profileIconId} color={a.color} size={28} label={a.label} />
        <span class="grow ellipsis">
          {nameOf(a)}
          {#if a.label && a.gameName}<span class="faint small">{riotId(a)}</span>{:else if a.tagLine}<span class="faint small">#{a.tagLine}</span>{/if}
        </span>
        {#if a.id === app.status.activeAccountId}<span class="chip accent">{t("configs.connected")}</span>{/if}
        {#if draft?.game === "val" && !dirOf(a)}<span class="faint small">{t("configs.noValDir")}</span>{/if}
        <span class="faint small">{a.region}</span>
      </label>
    {/each}
  </div>
  <p class="faint small" style="margin-top:8px">{t("configs.backupHint")}</p>
  {#snippet footer()}
    <button class="btn" onclick={() => (applyOpen = false)}>{t("common.cancel")}</button>
    <button class="btn primary" onclick={doApply} disabled={busy}><Icon name="play" size={13} />{t("configs.applyNow")}</button>
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
  .games {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 10px;
    margin-bottom: 12px;
  }
  .game {
    display: flex;
    align-items: center;
    gap: 10px;
    height: 48px;
    padding: 0 16px;
    border-radius: var(--radius-md);
    color: var(--text-2);
    font-size: 15px;
    font-weight: 650;
    cursor: pointer;
    text-align: start;
  }
  .game span:nth-child(2) {
    flex: 1;
  }
  .game.on {
    color: var(--text);
    border-color: var(--accent);
    background: var(--accent-soft);
  }
  .count {
    min-width: 22px;
    padding: 1px 7px;
    border-radius: 999px;
    background: var(--surface-2);
    font-size: 12px;
    text-align: center;
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
  .src {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    color: var(--text);
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
  .arow .faint {
    margin-left: 6px;
  }
</style>
