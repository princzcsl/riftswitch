<script lang="ts">
  import { api } from "../lib/api";
  import { app } from "../lib/app.svelte";
  import { errorText, t } from "../lib/i18n.svelte";
  import { ACCOUNT_COLORS } from "../lib/avatars";
  import Icon from "./Icon.svelte";
  import Modal from "./Modal.svelte";

  let { open = $bindable(false) }: { open?: boolean } = $props();
  let name = $state("");

  async function add() {
    if (!name.trim()) return;
    try {
      await api.saveFolder({ id: "", name, color: ACCOUNT_COLORS[app.folders.length % ACCOUNT_COLORS.length], order: 0 });
      name = "";
      await app.refreshAccounts();
    } catch (e) {
      app.toast("error", errorText(e));
    }
  }

  async function rename(id: string, value: string) {
    const f = app.folders.find((x) => x.id === id);
    if (!f || !value.trim() || value === f.name) return;
    await api.saveFolder({ ...$state.snapshot(f), name: value });
    await app.refreshAccounts();
  }

  async function recolor(id: string, color: string) {
    const f = app.folders.find((x) => x.id === id);
    if (!f) return;
    await api.saveFolder({ ...$state.snapshot(f), color });
    await app.refreshAccounts();
  }

  async function move(index: number, dir: -1 | 1) {
    const ids = app.folders.map((f) => f.id);
    const j = index + dir;
    if (j < 0 || j >= ids.length) return;
    [ids[index], ids[j]] = [ids[j], ids[index]];
    await api.reorderFolders(ids);
    await app.refreshAccounts();
  }

  async function remove(id: string) {
    await api.deleteFolder(id);
    await app.refreshAccounts();
  }
</script>

<Modal bind:open title={t("folders.title")} width={460}>
  <div class="row">
    <input class="input grow" placeholder={t("folders.newPh")} bind:value={name} onkeydown={(e) => e.key === "Enter" && add()} />
    <button class="btn primary" onclick={add}><Icon name="plus" size={16} />{t("common.add")}</button>
  </div>
  <div class="list">
    {#each app.folders as f, i (f.id)}
      <div class="item">
        <input type="color" class="dot-color" value={f.color || "#888888"} onchange={(e) => recolor(f.id, e.currentTarget.value)} />
        <input class="input grow" value={f.name} onchange={(e) => rename(f.id, e.currentTarget.value)} />
        <span class="faint small">{app.accounts.filter((a) => a.folderId === f.id).length}</span>
        <button class="btn ghost icon sm" onclick={() => move(i, -1)} disabled={i === 0} aria-label="up">
          <span style="display:grid;transform:rotate(180deg)"><Icon name="chevron" size={14} stroke={2} /></span>
        </button>
        <button class="btn ghost icon sm" onclick={() => move(i, 1)} disabled={i === app.folders.length - 1} aria-label="down">
          <Icon name="chevron" size={14} stroke={2} />
        </button>
        <button class="btn ghost icon sm" onclick={() => remove(f.id)} title={t("common.delete")}><Icon name="trash" size={15} /></button>
      </div>
    {:else}
      <p class="faint small" style="text-align:center;padding:16px">{t("folders.empty")}</p>
    {/each}
  </div>
</Modal>

<style>
  .list {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-top: 14px;
  }
  .item {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .dot-color {
    width: 22px;
    height: 22px;
    border: none;
    padding: 0;
    border-radius: 50%;
    background: none;
    cursor: pointer;
  }
</style>
