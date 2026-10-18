<script lang="ts">
  let {
    checked = $bindable(false),
    disabled = false,
    label = "",
    onchange,
  }: { checked?: boolean; disabled?: boolean; label?: string; onchange?: (v: boolean) => void } = $props();
</script>

<button
  type="button"
  role="switch"
  aria-checked={checked}
  aria-label={label}
  class="toggle"
  class:on={checked}
  {disabled}
  onclick={() => {
    checked = !checked;
    onchange?.(checked);
  }}
>
  <span class="knob"></span>
</button>

<style>
  .toggle {
    position: relative;
    width: 40px;
    height: 23px;
    flex: none;
    border-radius: 999px;
    border: 1px solid var(--border);
    background: var(--surface-hover);
    cursor: pointer;
    padding: 0;
    transition: background 0.2s;
  }
  .toggle.on {
    background: linear-gradient(135deg, var(--accent), var(--accent-2));
    border-color: transparent;
  }
  .toggle:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  .knob {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 17px;
    height: 17px;
    border-radius: 50%;
    background: #fff;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
    transition: transform 0.2s var(--ease);
  }
  .on .knob {
    transform: translateX(17px);
  }
</style>
