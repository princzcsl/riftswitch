<script lang="ts">
  // Fiche secondaire d'un compte. Les secrets (email, identifiant, mot de passe) sont en écriture seule :
  // le front ne les relit jamais, il n'affiche que des indicateurs et l'email masqué.
  import type { Account, AccountInput, Rank } from "../lib/types";
  import { api } from "../lib/api";
  import { app } from "../lib/app.svelte";
  import { errorText, t } from "../lib/i18n.svelte";
  import { ACCOUNT_COLORS } from "../lib/avatars";
  import { GAME_LOCALES, REGIONS, VAL_TIERS, hours } from "../lib/format";
  import Avatar from "./Avatar.svelte";
  import Icon from "./Icon.svelte";
  import IconPicker from "./IconPicker.svelte";
  import Modal from "./Modal.svelte";
  import TagInput from "./TagInput.svelte";
  import Toggle from "./Toggle.svelte";

  let { open = $bindable(false), account = null }: { open?: boolean; account?: Account | null } = $props();

  let tab = $state<"general" | "game" | "security" | "history">("general");
  let form = $state<AccountInput>(blank());
  let poolText = $state("");
  let agentText = $state("");
  let email = $state("");
  let username = $state("");
  let password = $state("");
  let clearPassword = $state(false);
  let valRank = $state<Rank>({ tier: "", division: "", lp: 0, wins: 0, losses: 0 });
  let showIcons = $state(false);
  let saving = $state(false);

  function blank(): AccountInput {
    return {
      id: null,
      label: "",
      gameName: "",
      tagLine: "",
      region: "EUW",
      color: ACCOUNT_COLORS[app.accounts.length % ACCOUNT_COLORS.length],
      icon: `glyph:${app.accounts.length % 12}`,
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
    };
  }

  // Réinitialise le formulaire à chaque ouverture. On ne LIT que `open` et `account` : l'effet ne
  // doit jamais relire `form` (qu'il écrit), sinon il boucle et remet l'onglet sur « Général ».
  let lastOpen = false;
  $effect(() => {
    if (!open) {
      lastOpen = false;
      return;
    }
    // svelte-ignore state_referenced_locally
    if (lastOpen) return; // déjà initialisé pour cette ouverture
    lastOpen = true;
    tab = "general";
    email = username = password = "";
    clearPassword = false;
    showIcons = false;
    let next: AccountInput;
    if (account) {
      const a = $state.snapshot(account) as Account;
      next = {
        id: a.id,
        label: a.label,
        gameName: a.gameName,
        tagLine: a.tagLine,
        region: a.region || "EUW",
        color: a.color,
        icon: a.icon,
        tags: a.tags,
        folderId: a.folderId,
        favorite: a.favorite,
        langs: { ...a.langs },
        twoFa: a.twoFa,
        notes: a.notes,
        championPool: a.championPool,
        agentPool: a.agentPool,
        roles: a.roles,
        profiles: { ...a.profiles },
      };
      valRank = a.ranks.val ?? { tier: "", division: "", lp: 0, wins: 0, losses: 0 };
    } else {
      next = blank();
      valRank = { tier: "", division: "", lp: 0, wins: 0, losses: 0 };
    }
    form = next;
    poolText = next.championPool.join(", ");
    agentText = next.agentPool.join(", ");
  });

  const list = (s: string) =>
    s
      .split(",")
      .map((x) => x.trim())
      .filter(Boolean);

  const ROLES = ["top", "jungle", "mid", "adc", "support"];
  const VAL_ROLES = ["duelist", "controller", "initiator", "sentinel"];

  function riotIdInput(v: string) {
    const [name, tag] = v.split("#");
    form.gameName = (name ?? "").trim();
    form.tagLine = (tag ?? "").trim();
  }

  async function save() {
    saving = true;
    try {
      const input: AccountInput = {
        ...$state.snapshot(form),
        tags: form.tags,
        championPool: list(poolText),
        agentPool: list(agentText),
        email: email ? email : null,
        username: username ? username : null,
        password: clearPassword ? "" : password ? password : null,
      };
      if (account && valRank.tier !== (account.ranks.val?.tier ?? "")) {
        input.ranks = { ...$state.snapshot(account.ranks), val: valRank.tier ? $state.snapshot(valRank) : null, manual: true };
      } else if (!account && valRank.tier) {
        input.ranks = { lolSolo: null, lolFlex: null, tft: null, val: $state.snapshot(valRank), updatedAt: null, manual: true };
      }
      const hadCreds = !!username || (!!password && !clearPassword);
      await api.saveAccount(input);
      password = "";
      app.toast("success", hadCreds ? t("editor.credentialsSaved") : t("editor.saved"));
      open = false;
      app.refreshAccounts();
    } catch (e) {
      app.toast("error", errorText(e));
    } finally {
      saving = false;
    }
  }

  const lolProfiles = $derived(app.profiles.filter((p) => p.game === "lol"));
  const valProfiles = $derived(app.profiles.filter((p) => p.game === "val"));
</script>

<Modal bind:open title={account ? t("editor.editTitle") : t("editor.newTitle")} width={680}>
  <div class="head">
    <button class="avatar-btn" onclick={() => (showIcons = !showIcons)} title={t("icon.change")}>
      <Avatar icon={form.icon} color={form.color} size={72} label={form.label} />
      <span class="edit-badge"><Icon name="image" size={13} /></span>
    </button>
    <div class="grow col" style="gap:8px">
      <input class="input title-input" placeholder={t("editor.labelPh")} bind:value={form.label} maxlength="40" />
      <div class="colors">
        {#each ACCOUNT_COLORS as c}
          <button class="swatch" class:sel={form.color === c} style="background:{c}" onclick={() => (form.color = c)} aria-label={c}></button>
        {/each}
        <input type="color" class="swatch native" bind:value={form.color} title={t("editor.customColor")} />
      </div>
    </div>
  </div>

  {#if showIcons}
    <div class="icons glass">
      <IconPicker bind:value={form.icon} color={form.color} profileIconId={account?.profileIconId ?? null} />
    </div>
  {/if}

  <div class="tabs">
    {#each ["general", "game", "security", "history"] as k}
      <button class:on={tab === k} onclick={() => (tab = k as typeof tab)}>{t(`editor.tab_${k}`)}</button>
    {/each}
  </div>

  {#if tab === "general"}
    <div class="grid2">
      <div class="field">
        <label for="rid">{t("editor.riotId")}</label>
        <input
          id="rid"
          class="input"
          placeholder="Name#TAG"
          value={form.gameName ? `${form.gameName}${form.tagLine ? "#" + form.tagLine : ""}` : ""}
          onchange={(e) => riotIdInput(e.currentTarget.value)}
        />
      </div>
      <div class="field">
        <label for="reg">{t("editor.region")}</label>
        <select id="reg" class="input" bind:value={form.region}>
          <option value="">{t("add.regionAuto")}</option>
          {#each REGIONS as r}<option value={r}>{r}</option>{/each}
        </select>
      </div>
      <div class="field">
        <label for="fld">{t("editor.folder")}</label>
        <select id="fld" class="input" bind:value={form.folderId}>
          <option value={null}>{t("editor.noFolder")}</option>
          {#each app.folders as f}<option value={f.id}>{f.name}</option>{/each}
        </select>
      </div>
      <div class="field">
        <label for="tags">{t("editor.tags")}</label>
        <TagInput bind:tags={form.tags} max={5} placeholder={t("add.tagsPh")} />
      </div>
    </div>
    <label class="row check"><Toggle bind:checked={form.favorite} /> {t("editor.favorite")}</label>
  {:else if tab === "game"}
    <div class="section-title" style="margin-top:4px">{t("editor.gameLangs")}</div>
    <div class="grid3">
      {#each [["lol", "League of Legends"], ["tft", "TFT"], ["val", "VALORANT"]] as [g, name]}
        <div class="field">
          <label for="lang-{g}">{name}</label>
          <select id="lang-{g}" class="input" bind:value={form.langs[g as "lol" | "tft" | "val"]}>
            <option value={null}>{t("editor.langDefault")}</option>
            {#each GAME_LOCALES as [code, n]}<option value={code}>{n}</option>{/each}
          </select>
        </div>
      {/each}
    </div>
    <div class="section-title">{t("editor.profiles")}</div>
    <div class="grid2">
      <div class="field">
        <label for="pl">{t("editor.profileLol")}</label>
        <select id="pl" class="input" bind:value={form.profiles.lol}>
          <option value={null}>{t("editor.none")}</option>
          {#each lolProfiles as p}<option value={p.id}>{p.name}</option>{/each}
        </select>
      </div>
      <div class="field">
        <label for="pv">{t("editor.profileVal")}</label>
        <select id="pv" class="input" bind:value={form.profiles.val}>
          <option value={null}>{t("editor.none")}</option>
          {#each valProfiles as p}<option value={p.id}>{p.name}</option>{/each}
        </select>
      </div>
    </div>
    <div class="section-title">{t("editor.roles")}</div>
    {#each [["LoL", ROLES], ["VAL", VAL_ROLES]] as [game, list]}
      <div class="row" style="flex-wrap:wrap;gap:6px;margin-bottom:6px">
        <span class="faint small" style="width:30px;font-weight:700">{game}</span>
        {#each list as r}
          <button
            class="chip"
            class:accent={form.roles.includes(r)}
            style="cursor:pointer"
            onclick={() => (form.roles = form.roles.includes(r) ? form.roles.filter((x) => x !== r) : [...form.roles, r])}
            >{t(`roles.${r}`)}</button
          >
        {/each}
      </div>
    {/each}
    <div class="grid2" style="margin-top:12px">
      <div class="field">
        <label for="pool">{t("editor.championPool")}</label>
        <input id="pool" class="input" placeholder="Ahri, Lux, Jinx" bind:value={poolText} />
      </div>
      <div class="field">
        <label for="agents">{t("editor.agentPool")}</label>
        <input id="agents" class="input" placeholder="Jett, Omen" bind:value={agentText} />
      </div>
    </div>
    <div class="section-title">{t("editor.valRank")}</div>
    <div class="grid3">
      <select class="input" bind:value={valRank.tier}>
        <option value="">{t("ranks.unranked")}</option>
        {#each VAL_TIERS as tier}<option value={tier}>{t(`tiers.${tier.toLowerCase()}`)}</option>{/each}
      </select>
      <select class="input" bind:value={valRank.division} disabled={!valRank.tier || valRank.tier === "RADIANT"}>
        <option value="1">1</option><option value="2">2</option><option value="3">3</option>
      </select>
      <input class="input" type="number" min="0" max="100" bind:value={valRank.lp} placeholder="RR" />
    </div>
    <p class="faint small">{t("editor.valRankHint")}</p>
  {:else if tab === "security"}
    <div class="note">
      <Icon name="shield" size={16} />
      <span>{t("editor.securityNote")}</span>
    </div>
    {#if account}
      {@const ready = account.hasUsername && account.hasPassword}
      <div class="loginState {ready ? 'ok' : 'off'}">
        <Icon name={ready ? "check" : "info"} size={16} />
        <span>{ready ? t("editor.loginReady") : t("editor.loginNeeded")}</span>
      </div>
    {/if}
    <div class="grid2">
      <div class="field">
        <label for="em">{t("editor.email")}</label>
        <input id="em" class="input" type="email" autocomplete="off" placeholder={account?.maskedEmail || "name@mail.com"} bind:value={email} />
      </div>
      <div class="field">
        <label class="lbl-row" for="un">{t("editor.username")}{#if account?.hasUsername}<span class="saved"><Icon name="check" size={11} />{t("editor.storedBadge")}</span>{/if}</label>
        <input id="un" class="input" autocomplete="off" placeholder={account?.hasUsername ? t("editor.keepCurrent") : t("editor.usernamePh")} bind:value={username} />
      </div>
      <div class="field">
        <label class="lbl-row" for="pw">{t("editor.password")}{#if account?.hasPassword}<span class="saved"><Icon name="check" size={11} />{t("editor.storedBadge")}</span>{/if}</label>
        <input
          id="pw"
          class="input"
          type="password"
          autocomplete="new-password"
          placeholder={account?.hasPassword ? t("editor.keepCurrent") : t("editor.optional")}
          bind:value={password}
          disabled={clearPassword}
        />
      </div>
      <div class="field" style="justify-content:flex-end">
        {#if account?.hasPassword}
          <label class="row check small"><input type="checkbox" bind:checked={clearPassword} /> {t("editor.removePassword")}</label>
        {/if}
      </div>
    </div>
    <p class="faint small">{t("editor.passwordHint")}</p>
    <label class="row check"><Toggle bind:checked={form.twoFa} /> {t("editor.twoFa")}</label>
    {#if !form.twoFa}<p class="small" style="color:var(--warning)">{t("editor.twoFaReminder")}</p>{/if}
    <div class="field" style="margin-top:14px">
      <label for="notes">{t("editor.notes")}</label>
      <textarea id="notes" class="input" placeholder={t("editor.notesPh")} bind:value={form.notes}></textarea>
    </div>
  {:else}
    {#if account}
      {@const p = account.playtime}
      {@const rows = [["League of Legends", p.lol, p.launcher?.lol ?? 0], ["TFT", p.tft, p.launcher?.tft ?? 0], ["VALORANT", p.val, p.launcher?.val ?? 0]] as const}
      <table class="hours">
        <thead>
          <tr><th></th><th>{t("editor.hoursInGame")}</th><th>{t("editor.hoursLauncher")}</th><th>{t("editor.hoursTotal")}</th></tr>
        </thead>
        <tbody>
          {#each rows as [name, game, launcher]}
            <tr><td>{name}</td><td>{hours(game)}</td><td>{hours(launcher)}</td><td><strong>{hours(game + launcher)}</strong></td></tr>
          {/each}
        </tbody>
        <tfoot>
          <tr>
            <td>{t("editor.hoursTotal")}</td>
            <td>{hours(p.lol + p.tft + p.val)}</td>
            <td>{hours(rows.reduce((s, r) => s + r[2], 0))}</td>
            <td><strong>{hours(rows.reduce((s, r) => s + r[1] + r[2], 0))}</strong></td>
          </tr>
        </tfoot>
      </table>
      <div class="section-title">{t("editor.aliases")}</div>
      {#each account.aliases as al}
        <div class="row small"><span>{al.riotId}</span><span class="spacer"></span><span class="faint">{new Date(al.seenAt).toLocaleDateString()}</span></div>
      {:else}
        <p class="faint small">{t("editor.noAliases")}</p>
      {/each}
      <div class="section-title">{t("editor.info")}</div>
      <p class="small muted">{t("editor.addedOn", { date: new Date(account.createdAt).toLocaleDateString() })}</p>
      {#if account.health.honorLevel != null}<p class="small muted">{t("editor.honor", { n: account.health.honorLevel })}</p>{/if}
    {/if}
  {/if}

  {#snippet footer()}
    <button class="btn" onclick={() => (open = false)}>{t("common.cancel")}</button>
    <button class="btn primary" onclick={save} disabled={saving}>{t("common.save")}</button>
  {/snippet}
</Modal>

<style>
  .head {
    display: flex;
    gap: 16px;
    align-items: center;
    margin-bottom: 14px;
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
  .title-input {
    font-size: 17px;
    font-weight: 650;
    height: 42px;
  }
  .colors {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
  }
  .swatch {
    width: 22px;
    height: 22px;
    border-radius: 50%;
    border: 2px solid transparent;
    cursor: pointer;
    padding: 0;
  }
  .swatch.sel {
    border-color: var(--text);
  }
  .swatch.native {
    background: conic-gradient(red, yellow, lime, aqua, blue, magenta, red);
    overflow: hidden;
    -webkit-appearance: none;
    appearance: none;
  }
  .swatch.native::-webkit-color-swatch {
    opacity: 0;
  }
  .icons {
    padding: 12px;
    margin-bottom: 14px;
    border-radius: var(--radius-md);
  }
  .tabs {
    display: flex;
    gap: 2px;
    border-bottom: 1px solid var(--border);
    margin-bottom: 14px;
  }
  .tabs button {
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
  .check {
    margin-top: 14px;
    cursor: pointer;
  }
  .note {
    display: flex;
    gap: 10px;
    align-items: flex-start;
    padding: 10px 12px;
    border-radius: var(--radius-sm);
    background: var(--accent-soft);
    color: var(--text-2);
    font-size: 12.5px;
    margin-bottom: 14px;
  }
  .loginState {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 9px 12px;
    border-radius: var(--radius-sm);
    font-size: 12.5px;
    font-weight: 600;
    margin-bottom: 14px;
  }
  .loginState.ok {
    background: color-mix(in srgb, var(--success) 15%, transparent);
    color: var(--success);
  }
  .loginState.off {
    background: var(--surface-hover);
    color: var(--text-2);
  }
  .lbl-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .saved {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    font-size: 10.5px;
    font-weight: 700;
    color: var(--success);
    text-transform: none;
    letter-spacing: 0;
  }
  .hours {
    width: 100%;
    border-collapse: collapse;
    font-size: 13px;
    margin-bottom: 6px;
  }
  .hours th {
    text-align: end;
    font-weight: 600;
    color: var(--text-3);
    font-size: 11px;
    padding: 0 8px 6px;
  }
  .hours td {
    text-align: end;
    padding: 8px;
    border-top: 1px solid var(--border);
  }
  .hours td:first-child {
    text-align: start;
    color: var(--text-2);
  }
  .hours tfoot td {
    border-top: 1px solid color-mix(in srgb, var(--accent) 40%, var(--border));
    font-weight: 600;
  }
</style>
