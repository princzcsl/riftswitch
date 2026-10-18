<script lang="ts">
  // Saisie de tags : Entrée (ou Tab) valide le tag, pas de virgule. Retour arrière sur champ vide retire le dernier.
  import Icon from "./Icon.svelte";

  let { tags = $bindable<string[]>([]), placeholder = "", max = 12 }: { tags?: string[]; placeholder?: string; max?: number } = $props();
  let draft = $state("");

  function add() {
    const v = draft.trim();
    if (v && !tags.includes(v) && tags.length < max) tags = [...tags, v];
    draft = "";
  }
  function onkey(e: KeyboardEvent) {
    if (e.key === "Enter" || e.key === "Tab") {
      if (draft.trim()) {
        e.preventDefault();
        add();
      }
    } else if (e.key === "Backspace" && !draft && tags.length) {
      tags = tags.slice(0, -1);
    }
  }
</script>

<div class="taginput input">
  {#each tags as tag}
    <span class="tag">{tag}<button type="button" onclick={() => (tags = tags.filter((t) => t !== tag))} aria-label="x"><Icon name="x" size={11} /></button></span>
  {/each}
  <input bind:value={draft} onkeydown={onkey} onblur={add} {placeholder} />
</div>

<style>
  .taginput {
    display: flex;
    flex-wrap: wrap;
    gap: 5px;
    align-items: center;
    height: auto;
    min-height: 38px;
    padding: 5px 8px;
  }
  .taginput input {
    flex: 1;
    min-width: 80px;
    border: none;
    outline: none;
    background: transparent;
    color: var(--text);
    height: 26px;
    user-select: text;
  }
  .tag {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    height: 24px;
    padding: 0 4px 0 9px;
    border-radius: var(--radius-pill);
    background: var(--accent-soft);
    color: var(--accent);
    font-size: 12px;
    font-weight: 650;
  }
  .tag button {
    display: grid;
    place-items: center;
    width: 16px;
    height: 16px;
    border: none;
    background: none;
    color: inherit;
    cursor: pointer;
    opacity: 0.7;
    padding: 0;
  }
  .tag button:hover {
    opacity: 1;
  }
</style>
