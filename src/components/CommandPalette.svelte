<script lang="ts">
  // Palette Ctrl+K : lancer un compte sur un jeu, ouvrir une page ou déclencher une action au clavier.
  import type { Account, Game } from "../lib/types";
  import { app, type Page } from "../lib/app.svelte";
  import { t } from "../lib/i18n.svelte";
  import Avatar from "./Avatar.svelte";
  import GameIcon from "./GameIcon.svelte";
  import Icon from "./Icon.svelte";

  type Item = { id: string; label: string; hint: string; keys: string; run: () => void; account?: Account; game?: Game; icon?: string };

  let open = $state(false);
  let query = $state("");
  let index = $state(0);
  let input = $state<HTMLInputElement>();

  const GAMES: Game[] = ["lol", "tft", "val"];
  const PAGES: { page: Page; icon: string }[] = [
    { page: "accounts", icon: "users" },
    { page: "tonight", icon: "moon" },
    { page: "configs", icon: "keyboard" },
    { page: "runes", icon: "hex" },
    { page: "shops", icon: "shop" },
    { page: "patches", icon: "scroll" },
    { page: "esport", icon: "trophy" },
    { page: "achievements", icon: "trophy" },
    { page: "settings", icon: "settings" },
  ];

  // Minuscules sans accents, pour chercher « paramètres » en tapant « parametres ».
  const norm = (s: string) => s.normalize("NFD").replace(/[̀-ͯ]/g, "").toLowerCase();

  const items = $derived.by<Item[]>(() => {
    const list: Item[] = [];
    for (const a of app.accounts) {
      const name = a.label || a.gameName || "?";
      for (const g of GAMES) {
        list.push({
          id: `${a.id}:${g}`,
          label: name,
          hint: t(`games.${g}`),
          keys: norm([name, a.gameName, a.tagLine, a.region, ...a.tags, t(`games.${g}`), t(`games.short_${g}`), g].join(" ")),
          run: () => app.play(a, g, app.preferOffline),
          account: a,
          game: g,
        });
      }
    }
    for (const p of PAGES) {
      list.push({ id: `page:${p.page}`, label: t(`nav.${p.page}`), hint: t("palette.page"), keys: norm(t(`nav.${p.page}`)), run: () => (app.page = p.page), icon: p.icon });
    }
    {
      const on = !app.preferOffline;
      list.push({ id: "ghost", label: t(on ? "palette.ghostOn" : "palette.ghostOff"), hint: t("palette.action"), keys: norm(`${t("palette.ghostOn")} ${t("palette.ghostOff")} ghost fantome offline`), run: () => app.setGhost(on), icon: "ghost" });
    }
    if (app.boot?.vaultMode === "password") {
      list.push({ id: "lock", label: t("nav.lock"), hint: t("palette.action"), keys: norm(t("nav.lock")), run: () => app.lock(), icon: "lock" });
    }
    return list;
  });

  const results = $derived.by<Item[]>(() => {
    const words = norm(query).split(/\s+/).filter(Boolean);
    if (!words.length) {
      // Sans saisie : favoris puis comptes récents sur leur dernier jeu, puis les actions.
      const recent = [...app.accounts]
        .sort((a, b) => Number(b.favorite) - Number(a.favorite) || (b.lastUsedAt ?? 0) - (a.lastUsedAt ?? 0))
        .slice(0, 5)
        .map((a) => items.find((i) => i.id === `${a.id}:${(a.lastGame ?? "lol") as Game}`));
      return [...recent, ...items.filter((i) => i.id === "ghost" || i.id === "lock")].filter((i): i is Item => !!i);
    }
    return items.filter((i) => words.every((w) => i.keys.includes(w))).slice(0, 8);
  });

  $effect(() => {
    void query;
    index = 0;
  });

  function show() {
    if (!app.unlocked) return;
    query = "";
    open = true;
    queueMicrotask(() => input?.focus());
  }

  function run(item: Item | undefined) {
    if (!item) return;
    open = false;
    item.run();
  }

  function onGlobalKey(e: KeyboardEvent) {
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "k") {
      e.preventDefault();
      open ? (open = false) : show();
    }
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === "Escape") open = false;
    else if (e.key === "ArrowDown") index = Math.min(index + 1, results.length - 1);
    else if (e.key === "ArrowUp") index = Math.max(index - 1, 0);
    else if (e.key === "Enter") run(results[index]);
    else return;
    e.preventDefault();
  }
</script>

<svelte:window onkeydown={onGlobalKey} />

{#if open}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="backdrop" role="presentation" onclick={(e) => e.target === e.currentTarget && (open = false)}>
    <div class="palette glass-strong glass-solid fade-in" role="dialog" aria-label={t("palette.placeholder")}>
      <div class="search">
        <Icon name="search" size={18} />
        <input bind:this={input} bind:value={query} onkeydown={onKey} placeholder={t("palette.placeholder")} spellcheck="false" />
        <kbd>Esc</kbd>
      </div>
      {#if results.length}
        <ul>
          {#each results as item, i (item.id)}
            <li>
              <button class:active={i === index} onmouseenter={() => (index = i)} onclick={() => run(item)}>
                {#if item.account}
                  <Avatar icon={item.account.icon} autoIconId={item.account.profileIconId} color={item.account.color} size={28} label={item.account.label} />
                {:else}
                  <span class="ico"><Icon name={item.icon ?? "chevronRight"} size={16} /></span>
                {/if}
                <span class="grow ellipsis">{item.label}</span>
                {#if item.game}<GameIcon game={item.game} size={16} />{/if}
                <span class="hint">{item.hint}</span>
              </button>
            </li>
          {/each}
        </ul>
      {:else}
        <p class="empty">{t("palette.empty")}</p>
      {/if}
      <p class="tip">{t("palette.tip")}</p>
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 90;
    background: rgba(0, 0, 0, 0.35);
    display: flex;
    justify-content: center;
    align-items: flex-start;
    padding-top: 14vh;
  }
  .palette {
    width: min(560px, calc(100vw - 32px));
    border-radius: var(--radius-lg);
    overflow: hidden;
  }
  .search {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 14px 16px;
    border-bottom: 1px solid var(--border);
    color: var(--text-3);
  }
  .search input {
    flex: 1;
    border: none;
    outline: none;
    background: transparent;
    color: var(--text);
    font-size: 16px;
  }
  kbd {
    font-size: 11px;
    padding: 2px 6px;
    border-radius: 6px;
    border: 1px solid var(--border);
    color: var(--text-3);
  }
  ul {
    list-style: none;
    margin: 0;
    padding: 6px;
    max-height: 360px;
    overflow-y: auto;
  }
  li button {
    display: flex;
    align-items: center;
    gap: 12px;
    width: 100%;
    padding: 9px 10px;
    border: none;
    border-radius: var(--radius-sm);
    background: none;
    color: var(--text);
    text-align: start;
    cursor: pointer;
    font-weight: 520;
  }
  li button.active {
    background: var(--surface-hover);
  }
  .ico {
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    border-radius: 50%;
    background: var(--surface-2);
    color: var(--accent);
  }
  .hint {
    font-size: 12px;
    color: var(--text-3);
  }
  .tip {
    margin: 0;
    padding: 8px 16px;
    border-top: 1px solid var(--border);
    font-size: 11.5px;
    color: var(--text-3);
  }
  .empty {
    margin: 0;
    padding: 18px;
    color: var(--text-3);
    text-align: center;
  }
</style>
