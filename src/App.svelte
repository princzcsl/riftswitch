<script lang="ts">
  import { app } from "./lib/app.svelte";
  import { latestVersion } from "./lib/ddragon";
  import { t } from "./lib/i18n.svelte";
  import Sidebar from "./components/Sidebar.svelte";
  import TitleBar from "./components/TitleBar.svelte";
  import MfaPrompt from "./components/MfaPrompt.svelte";
  import PlayOverlay from "./components/PlayOverlay.svelte";
  import UpdateScreen from "./components/UpdateScreen.svelte";
  import ConfirmDialog from "./components/ConfirmDialog.svelte";
  import CommandPalette from "./components/CommandPalette.svelte";
  import { updater } from "./lib/updater.svelte";
  import Toasts from "./components/Toasts.svelte";
  import Accounts from "./pages/Accounts.svelte";
  import Achievements from "./pages/Achievements.svelte";
  import Configs from "./pages/Configs.svelte";
  import Esport from "./pages/Esport.svelte";
  import Patches from "./pages/Patches.svelte";
  import Runes from "./pages/Runes.svelte";
  import Settings from "./pages/Settings.svelte";
  import Setup from "./pages/Setup.svelte";
  import Shops from "./pages/Shops.svelte";
  import Tonight from "./pages/Tonight.svelte";
  import Unlock from "./pages/Unlock.svelte";

  let ready = $state(false);
  let failed = $state("");

  app
    .init()
    .then(() => (ready = true))
    .catch((e) => (failed = String(e?.message ?? e)));

  const view = $derived.by(() => {
    if (!app.boot) return "loading";
    if (!app.boot.vaultExists) return "setup";
    if (!app.unlocked) return "unlock";
    if (!app.settings?.setupDone) return "setup-resume";
    return "main";
  });

  // Badge « nouveau patch » dans la barre latérale (lu localement, aucune notification système).
  let patchBadge = $state(false);
  $effect(() => {
    if (view !== "main") return;
    latestVersion().then((v) => {
      try {
        const seen = localStorage.getItem("rs.patchSeen");
        patchBadge = !!seen && seen !== v;
        if (!seen) localStorage.setItem("rs.patchSeen", v);
      } catch {
        patchBadge = false;
      }
    });
  });
  $effect(() => {
    if (app.page === "patches") patchBadge = false;
  });

  // Mise à jour automatique dès le démarrage (avant même le déverrouillage), une fois par lancement.
  let updateChecked = false;
  $effect(() => {
    if (!app.boot || updateChecked) return;
    updateChecked = true;
    // Juste après une mise à jour : pas d'écran de recherche, un simple message.
    const previous = app.boot.version ? updater.justUpdated(app.boot.version) : null;
    if (previous) app.toast("success", t("update.done", { v: app.boot.version }), 6000);
    if (app.settings?.checkUpdates) updater.autoUpdate(!previous);
    else updater.skipStartup();
  });
</script>

<div class="app-bg"><div class="blob"></div></div>

<div class="shell">
  <TitleBar />
  <div class="body">
    {#if failed}
      <div class="center"><p class="error">{failed}</p></div>
    {:else if !ready || view === "loading"}
      <div class="center"><img class="pulse-logo" src="/logo.svg" alt="" width="72" height="72" /></div>
    {:else if view === "setup"}
      <Setup />
    {:else if view === "unlock"}
      <Unlock />
    {:else if view === "setup-resume"}
      <Setup initial={2} />
    {:else}
      <Sidebar badges={{ patches: patchBadge }} />
      <main>
        {#key app.page}
          <div class="page-wrap fade-in">
            {#if app.page === "accounts"}<Accounts />
            {:else if app.page === "tonight"}<Tonight />
            {:else if app.page === "configs"}<Configs />
            {:else if app.page === "runes"}<Runes />
            {:else if app.page === "shops"}<Shops />
            {:else if app.page === "patches"}<Patches />
            {:else if app.page === "esport"}<Esport />
            {:else if app.page === "achievements"}<Achievements />
            {:else if app.page === "settings"}<Settings />
            {/if}
          </div>
        {/key}
      </main>
    {/if}
  </div>
</div>

<MfaPrompt />
<PlayOverlay />
<UpdateScreen />
<ConfirmDialog />
<CommandPalette />
<Toasts />
<span class="sr-only" aria-live="polite">{app.status.busy ? t("status.switching") : ""}</span>

<style>
  .shell {
    height: 100%;
    display: flex;
    flex-direction: column;
  }
  .body {
    flex: 1;
    display: flex;
    min-height: 0;
  }
  main {
    flex: 1;
    min-width: 0;
    position: relative;
  }
  .page-wrap {
    height: 100%;
  }
  .center {
    flex: 1;
    display: grid;
    place-items: center;
  }
  .pulse-logo {
    animation: breathe 1.6s ease-in-out infinite;
  }
  @keyframes breathe {
    50% {
      transform: scale(1.06);
      opacity: 0.8;
    }
  }
  .error {
    color: var(--danger);
  }
  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip: rect(0 0 0 0);
  }
</style>
