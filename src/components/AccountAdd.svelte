<script lang="ts">
  // Fenêtre d'ajout de compte : design dédié (différent de l'éditeur à onglets « Modifier »).
  // Sections profil + Connexion (identifiant/mot de passe, nécessaires à la bascule en un clic).
  import type { AccountInput } from "../lib/types";
  import { api } from "../lib/api";
  import { app } from "../lib/app.svelte";
  import { errorText, t } from "../lib/i18n.svelte";
  import { ACCOUNT_COLORS, randomColor } from "../lib/avatars";
  import { GAME_LOCALES } from "../lib/format";
  import Avatar from "./Avatar.svelte";
  import Icon from "./Icon.svelte";
  import IconPicker from "./IconPicker.svelte";
  import Modal from "./Modal.svelte";
  import TagInput from "./TagInput.svelte";
  import Toggle from "./Toggle.svelte";

  let { open = $bindable(false) }: { open?: boolean } = $props();

  let tab = $state<"profile" | "langs">("profile");
  let showIcons = $state(false);
  let saving = $state(false);
  let newFolder = $state("");
  let creatingFolder = $state(false);

  const LOL_ROLES = ["top", "jungle", "mid", "adc", "support"];
  const VAL_ROLES = ["duelist", "controller", "initiator", "sentinel"];

  function blank(): AccountInput {
    return {
      id: null,
      label: "",
      gameName: "",
      tagLine: "",
      region: "",
      color: randomColor(app.accounts.map((a) => a.color)),
      icon: "auto",
      tags: [],
      folderId: null,
      favorite: false,
      langs: { lol: null, tft: null, val: null },
      twoFa: false,
      notes: "",
      championPool: [],
      agentPool: [],
      roles: [],
      profiles: { lol: null, val: null },
      email: "",
      username: "",
      password: "",
    };
  }

  let form = $state<AccountInput>(blank());

  // Réinitialise à chaque ouverture (sans boucle : on ne lit pas `form`).
  let wasOpen = false;
  $effect(() => {
    if (!open) {
      wasOpen = false;
      return;
    }
    // svelte-ignore state_referenced_locally
    if (wasOpen) return;
    wasOpen = true;
    form = blank();
    tab = "profile";
    showIcons = false;
    creatingFolder = false;
    newFolder = "";
  });

  function toggleRole(r: string) {
    form.roles = form.roles.includes(r) ? form.roles.filter((x) => x !== r) : [...form.roles, r];
  }

  async function createFolder() {
    const name = newFolder.trim();
    if (!name) return;
    try {
      const f = await api.saveFolder({ id: "", name, color: randomColor(app.folders.map((x) => x.color ?? "")), order: 0 });
      await app.refreshAccounts();
      form.folderId = f.id;
      newFolder = "";
      creatingFolder = false;
    } catch (e) {
      app.toast("error", errorText(e));
    }
  }

  async function save() {
    if (!form.username || !form.password) {
      tab = "profile";
      app.toast("warning", t("add.needCreds"));
      return;
    }
    saving = true;
    try {
      await api.saveAccount({ ...$state.snapshot(form), email: form.email || null });
      app.toast("success", t("add.done"));
      open = false;
      app.refreshAccounts();
    } catch (e) {
      app.toast("error", errorText(e));
    } finally {
      saving = false;
    }
  }
</script>

<Modal bind:open title={t("add.title")} width={620}>
  <div class="add">
    <!-- En-tête : icône + nom + couleurs -->
    <div class="hero">
      <button class="avatar-btn" onclick={() => (showIcons = !showIcons)} title={t("icon.change")}>
        <Avatar icon={form.icon} color={form.color} size={76} label={form.label} />
        <span class="edit-badge"><Icon name="image" size={13} /></span>
      </button>
      <div class="grow col" style="gap:10px">
        <input class="input name" placeholder={t("add.labelPh")} bind:value={form.label} maxlength="40" />
        <div class="colors">
          {#each ACCOUNT_COLORS.slice(0, 7) as c}
            <button class="swatch" class:sel={form.color === c} style="background:{c}" onclick={() => (form.color = c)} aria-label={c}></button>
          {/each}
          <label class="swatch native" style="background:{form.color}" title={t("editor.customColor")}>
            <Icon name="sparkle" size={13} />
            <input type="color" bind:value={form.color} />
          </label>
        </div>
      </div>
    </div>

    {#if showIcons && tab === "profile"}
      <div class="iconbox glass">
        <IconPicker bind:value={form.icon} color={form.color} profileIconId={null} />
      </div>
    {/if}

    <div class="segtabs">
      <button class:on={tab === "profile"} onclick={() => (tab = "profile")}><Icon name="users" size={15} />{t("add.tabProfile")}</button>
      <button class:on={tab === "langs"} onclick={() => (tab = "langs")}><Icon name="chevronRight" size={15} />{t("add.tabLangs")}</button>
    </div>

    {#if tab === "profile"}
      <!-- Connexion -->
      <div class="section">
        <div class="section-head"><Icon name="login" size={15} /><span>{t("add.login")}</span></div>
        <div class="grid2">
          <div class="field">
            <label for="au">{t("editor.username")}</label>
            <input id="au" class="input" autocomplete="off" placeholder={t("editor.usernamePh")} bind:value={form.username} />
          </div>
          <div class="field">
            <label for="ap">{t("editor.password")}</label>
            <input id="ap" class="input" type="password" autocomplete="new-password" placeholder="••••••••" bind:value={form.password} />
          </div>
        </div>
        <p class="faint small">{t("add.loginHint")}</p>
      </div>

      <!-- Classement -->
      <div class="section">
        <div class="section-head"><Icon name="folder" size={15} /><span>{t("add.organize")}</span></div>
        <div class="grid2">
          <div class="field">
            <label for="af">{t("editor.folder")}</label>
            {#if creatingFolder}
              <div class="row">
                <input class="input grow" placeholder={t("folders.newPh")} bind:value={newFolder} onkeydown={(e) => e.key === "Enter" && createFolder()} />
                <button class="btn sm primary" onclick={createFolder}>{t("common.add")}</button>
                <button class="btn sm ghost icon" onclick={() => (creatingFolder = false)} aria-label="x"><Icon name="x" size={15} /></button>
              </div>
            {:else}
              <div class="row">
                <select id="af" class="input grow" bind:value={form.folderId}>
                  <option value={null}>{t("editor.noFolder")}</option>
                  {#each app.folders as f}<option value={f.id}>{f.name}</option>{/each}
                </select>
                <button class="btn sm" onclick={() => (creatingFolder = true)} title={t("add.newFolder")}><Icon name="plus" size={15} /></button>
              </div>
            {/if}
          </div>
          <div class="field">
            <label for="ar">{t("editor.region")}</label>
            <div class="row">
              <select id="ar" class="input grow" bind:value={form.region}>
                <option value="">{t("add.regionAuto")}</option>
                {#each ["EUW", "EUNE", "NA", "KR", "JP", "BR", "LAN", "LAS", "OCE", "TR", "RU", "ME", "SEA", "TW", "VN"] as r}<option value={r}>{r}</option>{/each}
              </select>
              <label class="fav-toggle" title={t("editor.favorite")}><Toggle bind:checked={form.favorite} /><Icon name={form.favorite ? "starFill" : "star"} size={16} /></label>
            </div>
          </div>
        </div>
        <div class="field">
          <label for="at">{t("editor.tags")}</label>
          <TagInput bind:tags={form.tags} max={5} placeholder={t("add.tagsPh")} />
        </div>
      </div>

      <!-- Rôles -->
      <div class="section">
        <div class="section-head"><Icon name="target" size={15} /><span>{t("add.roles")}</span></div>
        <div class="roles">
          <span class="roles-label">LoL</span>
          {#each LOL_ROLES as r}
            <button class="chip" class:accent={form.roles.includes(r)} onclick={() => toggleRole(r)}>{t(`roles.${r}`)}</button>
          {/each}
        </div>
        <div class="roles">
          <span class="roles-label">VAL</span>
          {#each VAL_ROLES as r}
            <button class="chip" class:accent={form.roles.includes(r)} onclick={() => toggleRole(r)}>{t(`roles.${r}`)}</button>
          {/each}
        </div>
      </div>

      <!-- Note -->
      <div class="section">
        <div class="section-head"><Icon name="edit" size={15} /><span>{t("add.notesSection")}</span></div>
        <div class="field">
          <label for="ae">{t("add.emailNote")}</label>
          <input id="ae" class="input" type="email" autocomplete="off" placeholder="name@mail.com" bind:value={form.email} />
          <span class="faint small">{t("add.emailHint")}</span>
        </div>
        <div class="field">
          <label for="an">{t("editor.notes")}</label>
          <textarea id="an" class="input" placeholder={t("editor.notesPh")} bind:value={form.notes}></textarea>
        </div>
      </div>
    {:else}
      <div class="section">
        <div class="section-head"><Icon name="info" size={15} /><span>{t("editor.gameLangs")}</span></div>
        <p class="faint small">{t("add.langsHint")}</p>
        <div class="grid3">
          {#each [["lol", "League of Legends"], ["tft", "TFT"], ["val", "VALORANT"]] as [g, name]}
            <div class="field">
              <label for="al-{g}">{name}</label>
              <select id="al-{g}" class="input" bind:value={form.langs[g as "lol" | "tft" | "val"]}>
                <option value={null}>{t("editor.langDefault")}</option>
                {#each GAME_LOCALES as [code, n]}<option value={code}>{n}</option>{/each}
              </select>
            </div>
          {/each}
        </div>
      </div>
    {/if}
  </div>

  {#snippet footer()}
    <button class="btn" onclick={() => (open = false)}>{t("common.cancel")}</button>
    <button class="btn primary" onclick={save} disabled={saving}><Icon name="plus" size={15} />{t("add.create")}</button>
  {/snippet}
</Modal>

<style>
  .add {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }
  .hero {
    display: flex;
    gap: 16px;
    align-items: center;
    padding: 4px 2px;
  }
  .avatar-btn {
    position: relative;
    border: none;
    background: none;
    padding: 0;
    cursor: pointer;
  }
  .edit-badge {
    position: absolute;
    right: -2px;
    bottom: -2px;
    width: 26px;
    height: 26px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    background: var(--surface-2);
    border: 1px solid var(--border);
    color: var(--text);
  }
  .name {
    font-size: 18px;
    font-weight: 650;
    height: 44px;
  }
  .colors {
    display: flex;
    gap: 7px;
    flex-wrap: wrap;
    align-items: center;
  }
  .swatch {
    width: 24px;
    height: 24px;
    border-radius: 50%;
    border: 2px solid transparent;
    cursor: pointer;
    padding: 0;
  }
  .swatch.sel {
    border-color: var(--text);
  }
  .swatch.native {
    display: grid;
    place-items: center;
    color: rgba(255, 255, 255, 0.9);
    position: relative;
    overflow: hidden;
  }
  .swatch.native input {
    position: absolute;
    inset: 0;
    opacity: 0;
    cursor: pointer;
  }
  .iconbox {
    padding: 12px;
    border-radius: var(--radius-md);
  }
  .segtabs {
    display: inline-flex;
    gap: 3px;
    padding: 3px;
    border-radius: var(--radius-sm);
    background: var(--surface);
    border: 1px solid var(--border);
    align-self: flex-start;
  }
  .segtabs button {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    height: 30px;
    padding: 0 14px;
    border: none;
    border-radius: calc(var(--radius-sm) - 2px);
    background: transparent;
    color: var(--text-2);
    cursor: pointer;
    font-weight: 600;
  }
  .segtabs button.on {
    background: var(--surface-2);
    color: var(--text);
    box-shadow: 0 1px 4px rgba(0, 0, 0, 0.2);
  }
  .section {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 14px;
    border-radius: var(--radius-md);
    background: var(--surface);
    border: 1px solid var(--border);
  }
  .section-head {
    display: flex;
    align-items: center;
    gap: 8px;
    font-weight: 650;
    font-size: 13px;
  }
  .section-head :global(svg) {
    color: var(--accent);
  }
  .grid2 {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
  }
  .grid3 {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 10px;
  }
  .fav-toggle {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    color: #ffc94d;
    cursor: pointer;
  }
  .roles {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-wrap: wrap;
  }
  .roles-label {
    font-size: 11px;
    font-weight: 700;
    color: var(--text-3);
    width: 30px;
  }
  .roles .chip {
    cursor: pointer;
  }
</style>
