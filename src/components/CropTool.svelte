<script lang="ts">
  // Recadrage circulaire intégré : glisser pour déplacer, molette / curseur pour zoomer. Sortie PNG 160×160.
  import { t } from "../lib/i18n.svelte";

  let { src, ondone, oncancel }: { src: string; ondone: (dataUrl: string) => void; oncancel: () => void } = $props();

  const VIEW = 240;
  let img = $state<HTMLImageElement | null>(null);
  let scale = $state(1);
  let minScale = $state(1);
  let x = $state(0);
  let y = $state(0);
  let drag: { sx: number; sy: number; ox: number; oy: number } | null = null;

  $effect(() => {
    const i = new Image();
    i.onload = () => {
      img = i;
      minScale = Math.max(VIEW / i.width, VIEW / i.height);
      scale = minScale;
      x = (VIEW - i.width * scale) / 2;
      y = (VIEW - i.height * scale) / 2;
    };
    i.src = src;
  });

  function clamp() {
    if (!img) return;
    const w = img.width * scale;
    const h = img.height * scale;
    x = Math.min(0, Math.max(VIEW - w, x));
    y = Math.min(0, Math.max(VIEW - h, y));
  }

  function zoom(next: number) {
    if (!img) return;
    const s = Math.max(minScale, Math.min(minScale * 5, next));
    // Zoom centré sur le milieu du cadre.
    const cx = (VIEW / 2 - x) / scale;
    const cy = (VIEW / 2 - y) / scale;
    scale = s;
    x = VIEW / 2 - cx * s;
    y = VIEW / 2 - cy * s;
    clamp();
  }

  function done() {
    if (!img) return;
    const out = 160;
    const c = document.createElement("canvas");
    c.width = out;
    c.height = out;
    const ctx = c.getContext("2d")!;
    ctx.beginPath();
    ctx.arc(out / 2, out / 2, out / 2, 0, Math.PI * 2);
    ctx.clip();
    const k = out / VIEW;
    ctx.drawImage(img, x * k, y * k, img.width * scale * k, img.height * scale * k);
    ondone(c.toDataURL("image/png"));
  }
</script>

<div class="crop col">
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="frame"
    style="width:{VIEW}px;height:{VIEW}px"
    onpointerdown={(e) => {
      drag = { sx: e.clientX, sy: e.clientY, ox: x, oy: y };
      (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    }}
    onpointermove={(e) => {
      if (!drag) return;
      x = drag.ox + e.clientX - drag.sx;
      y = drag.oy + e.clientY - drag.sy;
      clamp();
    }}
    onpointerup={() => (drag = null)}
    onwheel={(e) => {
      e.preventDefault();
      zoom(scale * (e.deltaY < 0 ? 1.08 : 0.93));
    }}
  >
    {#if img}
      <img {src} alt="" draggable="false" style="left:{x}px;top:{y}px;width:{img.width * scale}px;height:{img.height * scale}px" />
    {/if}
    <div class="mask"></div>
  </div>
  <input type="range" min={minScale} max={minScale * 5} step="0.01" value={scale} oninput={(e) => zoom(Number(e.currentTarget.value))} />
  <div class="row" style="justify-content:flex-end">
    <button class="btn" onclick={oncancel}>{t("common.cancel")}</button>
    <button class="btn primary" onclick={done}>{t("common.apply")}</button>
  </div>
</div>

<style>
  .crop {
    align-items: center;
  }
  .frame {
    position: relative;
    overflow: hidden;
    border-radius: var(--radius-md);
    cursor: grab;
    background: #000;
    touch-action: none;
  }
  img {
    position: absolute;
    max-width: none;
    pointer-events: none;
  }
  .mask {
    position: absolute;
    inset: 0;
    border-radius: 50%;
    box-shadow: 0 0 0 999px rgba(0, 0, 0, 0.55);
    outline: 2px solid rgba(255, 255, 255, 0.8);
    pointer-events: none;
  }
  input[type="range"] {
    width: 240px;
    accent-color: var(--accent);
  }
</style>
