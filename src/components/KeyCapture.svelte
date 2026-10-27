<script lang="ts">
  // Case de touche : clic → « appuie sur une touche » ; Échap annule, Retour arrière / Suppr délie.
  // Mode LoL : renvoie `[Ctrl][Shift][q]` (positions QWERTY). Mode Valorant : nom Unreal + modificateurs.
  import { LOL_UNBOUND, lolCombo, lolKeyName, lolMouseName, prettyLol, prettyVal, valKeyName, valMouseName, type ValKey } from "../lib/keys";
  import { t } from "../lib/i18n.svelte";

  let {
    mode,
    value = "",
    valKey = null,
    conflict = false,
    onlol,
    onval,
  }: {
    mode: "lol" | "val";
    value?: string;
    valKey?: ValKey | null;
    conflict?: boolean;
    onlol?: (v: string) => void;
    onval?: (v: ValKey | null) => void;
  } = $props();

  let capturing = $state(false);
  let startedAt = 0;

  const label = $derived.by(() => {
    if (capturing) return t("keys.press");
    if (mode === "lol") return prettyLol(value) || "—";
    if (!valKey || !valKey.key || valKey.key === "None") return "—";
    const mods = [valKey.ctrl && "Ctrl", valKey.alt && "Alt", valKey.shift && "Shift"].filter(Boolean).join(" + ");
    return `${mods ? mods + " + " : ""}${prettyVal(valKey.key)}`;
  });

  function start(e: MouseEvent) {
    e.stopPropagation();
    capturing = true;
    startedAt = performance.now();
  }

  function stop() {
    capturing = false;
  }

  function emitUnbind() {
    if (mode === "lol") onlol?.(LOL_UNBOUND);
    else onval?.(null);
    stop();
  }

  function onKey(e: KeyboardEvent) {
    if (!capturing) return;
    e.preventDefault();
    e.stopPropagation();
    if (e.key === "Escape") return stop();
    if (e.key === "Backspace" || e.key === "Delete") return emitUnbind();
    const mods = { shift: e.shiftKey, ctrl: e.ctrlKey, alt: e.altKey };
    if (mode === "lol") {
      const k = lolKeyName(e.code);
      if (!k) return; // modificateur seul : on attend la touche
      onlol?.(lolCombo(k, mods));
    } else {
      const k = valKeyName(e);
      if (!k) return;
      const isMod = ["LeftShift", "RightShift", "LeftControl", "RightControl", "LeftAlt", "RightAlt"].includes(k);
      onval?.({ key: k, shift: isMod ? false : mods.shift, ctrl: isMod ? false : mods.ctrl, alt: isMod ? false : mods.alt });
    }
    stop();
  }

  function onMouse(e: MouseEvent) {
    if (!capturing) return;
    // Le clic qui a ouvert la capture ne compte pas.
    if (e.button === 0 && performance.now() - startedAt < 250) return;
    e.preventDefault();
    e.stopPropagation();
    const mods = { shift: e.shiftKey, ctrl: e.ctrlKey, alt: e.altKey };
    if (mode === "lol") {
      const b = lolMouseName(e.button);
      if (b) onlol?.(lolCombo(b, mods));
    } else {
      const b = valMouseName(e.button);
      if (b) onval?.({ key: b, ...mods });
    }
    stop();
  }

  function onWheel(e: WheelEvent) {
    if (!capturing || mode !== "val") return;
    e.preventDefault();
    onval?.({ key: e.deltaY < 0 ? "MouseScrollUp" : "MouseScrollDown", shift: e.shiftKey, ctrl: e.ctrlKey, alt: e.altKey });
    stop();
  }
</script>

<svelte:window onkeydowncapture={onKey} onmousedowncapture={onMouse} onwheel={onWheel} oncontextmenu={(e) => capturing && e.preventDefault()} onblur={stop} />

{#snippet mouse(b: string)}
  <svg class="mouse" viewBox="0 0 12 18" width="13" height="19" aria-label={t(`keys.mouse_${b}`)}>
    <rect x="1" y="1" width="10" height="16" rx="5" fill="none" stroke="currentColor" stroke-width="1.4" />
    <line x1="1" y1="7" x2="11" y2="7" stroke="currentColor" stroke-width="1.2" />
    <line x1="6" y1="1" x2="6" y2="7" stroke="currentColor" stroke-width="1.2" />
    {#if b === "L"}<path d="M6 1.6 A4.4 4.4 0 0 0 1.6 6 V7 H6 Z" fill="currentColor" />{/if}
    {#if b === "R"}<path d="M6 1.6 A4.4 4.4 0 0 1 10.4 6 V7 H6 Z" fill="currentColor" />{/if}
    {#if b === "M" || b === "↑" || b === "↓"}<rect x="4.8" y="2.5" width="2.4" height="3.5" rx="1.2" fill="currentColor" />{/if}
  </svg>
  {#if !["L", "R", "M"].includes(b)}<span class="mb">{b}</span>{/if}
{/snippet}

<span class="kc">
  <button type="button" class="slot" class:capturing class:empty={label === "—"} class:conflict onclick={start} title={conflict ? t("keys.conflict") : ""}>
    {#each label.split(/(🖱[^\s+]*)/) as part}
      {#if part.startsWith("🖱")}{@render mouse(part.slice(2))}{:else}{part}{/if}
    {/each}
  </button>
  {#if label !== "—" && !capturing}
    <button type="button" class="clear" title={t("keys.unbind")} aria-label={t("keys.unbind")} onclick={emitUnbind}>×</button>
  {/if}
</span>

<style>
  .kc {
    position: relative;
    display: inline-flex;
  }
  .clear {
    position: absolute;
    top: 50%;
    right: 4px;
    transform: translateY(-50%);
    width: 18px;
    height: 18px;
    border: none;
    border-radius: 50%;
    background: transparent;
    color: var(--text-3);
    font-size: 14px;
    line-height: 1;
    cursor: pointer;
    opacity: 0;
    transition: opacity 0.15s;
  }
  .kc:hover .clear {
    opacity: 1;
  }
  .clear:hover {
    background: var(--surface-2);
    color: var(--danger);
  }
  .mouse {
    vertical-align: -3px;
  }
  .mb {
    margin-left: 2px;
    font-size: 11px;
  }
  .slot {
    min-width: 108px;
    height: 32px;
    padding: 0 10px;
    border-radius: 8px;
    border: 1px solid var(--border);
    border-bottom-width: 2px;
    background: var(--surface-hover);
    color: var(--text);
    font-weight: 650;
    font-size: 12.5px;
    cursor: pointer;
    white-space: nowrap;
    transition:
      border-color 0.15s,
      background 0.15s;
  }
  .slot:hover {
    border-color: var(--text-3);
  }
  .slot.empty {
    color: var(--text-3);
    font-weight: 500;
  }
  .slot.conflict {
    border-color: var(--warning);
    color: var(--warning);
  }
  .slot.capturing {
    border-color: var(--accent);
    background: var(--accent-soft);
    color: var(--accent);
    animation: pulse 1.2s infinite;
  }
</style>
