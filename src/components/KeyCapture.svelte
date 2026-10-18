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

<button type="button" class="slot" class:capturing class:empty={label === "—"} class:conflict onclick={start} title={conflict ? t("keys.conflict") : ""}>
  {label}
</button>

<style>
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
