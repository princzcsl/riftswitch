<script lang="ts">
  import { GLYPHS } from "../lib/avatars";
  import { championIcon, profileIcon } from "../lib/ddragon";
  import { app } from "../lib/app.svelte";

  let {
    icon = "",
    color = "#7c5cff",
    size = 56,
    label = "",
    ring = false,
    autoIconId = null,
  }: { icon?: string; color?: string; size?: number; label?: string; ring?: boolean; autoIconId?: number | null } = $props();

  // "auto" (ou vide) : on utilise l'icône d'invocateur LoL récupérée pour le compte, si connue.
  const resolved = $derived(!icon || icon === "auto" ? (autoIconId ? `profile:${autoIconId}` : "auto") : icon);
  const kind = $derived(resolved.split(":")[0]);
  const value = $derived(resolved.slice(resolved.indexOf(":") + 1));
  const glyph = $derived(kind === "glyph" ? GLYPHS.find((g) => g.id === value) : undefined);
  const src = $derived(
    kind === "champ" && app.ddVersion
      ? championIcon(app.ddVersion, value)
      : kind === "profile" && app.ddVersion
        ? profileIcon(app.ddVersion, Number(value))
        : kind === "agent"
          ? `https://media.valorant-api.com/agents/${value}/displayicon.png`
          : kind === "data"
            ? resolved
            : "",
  );
</script>

<div
  class="avatar"
  class:ring
  style="--c:{color}; width:{size}px; height:{size}px; font-size:{Math.round(size * 0.4)}px"
  aria-label={label}
>
  {#if src}
    <img {src} alt="" draggable="false" />
  {:else if glyph}
    <svg viewBox="0 0 64 64" style="color:{color}">{@html glyph.svg}</svg>
  {:else}
    <span>{(label || "?").slice(0, 1).toUpperCase()}</span>
  {/if}
</div>

<style>
  .avatar {
    position: relative;
    flex: none;
    border-radius: 50%;
    display: grid;
    place-items: center;
    overflow: hidden;
    background: radial-gradient(circle at 30% 25%, color-mix(in srgb, var(--c) 70%, #fff 30%), var(--c) 55%, color-mix(in srgb, var(--c) 70%, #000 30%));
    box-shadow:
      inset 0 1px 1px rgba(255, 255, 255, 0.35),
      0 4px 14px color-mix(in srgb, var(--c) 40%, transparent);
    color: #fff;
    font-weight: 700;
  }
  .avatar.ring {
    outline: 2px solid var(--success);
    outline-offset: 2px;
  }
  img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  svg {
    width: 82%;
    height: 82%;
  }
</style>
