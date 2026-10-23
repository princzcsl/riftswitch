<script lang="ts">
  // Icônes trait (24×24), dessinées pour l'app : aucune dépendance externe.
  let { name, size = 18, stroke = 1.8 }: { name: string; size?: number; stroke?: number } = $props();

  const P: Record<string, string> = {
    users: `<circle cx="9" cy="8" r="3.5"/><path d="M2.5 20c.6-3.6 3.2-5.5 6.5-5.5s5.9 1.9 6.5 5.5"/><path d="M16 4.6a3.4 3.4 0 0 1 0 6.8M18.5 14.8c1.8.7 2.9 2.4 3.2 5"/>`,
    moon: `<path d="M20 14.5A8 8 0 1 1 9.5 4a6.5 6.5 0 0 0 10.5 10.5Z"/>`,
    keyboard: `<rect x="2.5" y="6" width="19" height="12" rx="2.5"/><path d="M6.5 10h.01M10 10h.01M13.5 10h.01M17 10h.01M7.5 14h9"/>`,
    hex: `<path d="M12 2.8 20 7.4v9.2l-8 4.6-8-4.6V7.4Z"/><circle cx="12" cy="12" r="3"/>`,
    shop: `<path d="M4 8h16l-1.2 11.2a2 2 0 0 1-2 1.8H7.2a2 2 0 0 1-2-1.8Z"/><path d="M8.5 8V6.5a3.5 3.5 0 0 1 7 0V8"/>`,
    scroll: `<path d="M7 3.5h10.5A2.5 2.5 0 0 1 20 6v0a2.5 2.5 0 0 1-2.5 2.5H17v10a2 2 0 0 1-2 2H6.5A2.5 2.5 0 0 1 4 18v0a2.5 2.5 0 0 1 2.5-2.5H7Z"/><path d="M10 8.5h4M10 12h4"/>`,
    trophy: `<path d="M7 4h10v4.5a5 5 0 0 1-10 0Z"/><path d="M7 6H4.5a2.5 2.5 0 0 0 2.6 3.2M17 6h2.5a2.5 2.5 0 0 1-2.6 3.2M12 13.5V17M8.5 20.5h7M9.5 17h5"/>`,
    award: `<circle cx="12" cy="9" r="5.5"/><path d="m8.6 13.4-1.6 7.1 5-2.6 5 2.6-1.6-7.1"/>`,
    settings: `<circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.6 1.6 0 0 0 .3 1.8l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.6 1.6 0 0 0-1.8-.3 1.6 1.6 0 0 0-1 1.5V21a2 2 0 1 1-4 0v-.1A1.6 1.6 0 0 0 9 19.4a1.6 1.6 0 0 0-1.8.3l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1a1.6 1.6 0 0 0 .3-1.8 1.6 1.6 0 0 0-1.5-1H3a2 2 0 1 1 0-4h.1A1.6 1.6 0 0 0 4.6 9a1.6 1.6 0 0 0-.3-1.8l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1a1.6 1.6 0 0 0 1.8.3H9a1.6 1.6 0 0 0 1-1.5V3a2 2 0 1 1 4 0v.1a1.6 1.6 0 0 0 1 1.5 1.6 1.6 0 0 0 1.8-.3l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.6 1.6 0 0 0-.3 1.8V9a1.6 1.6 0 0 0 1.5 1H21a2 2 0 1 1 0 4h-.1a1.6 1.6 0 0 0-1.5 1Z"/>`,
    lock: `<rect x="4.5" y="10.5" width="15" height="10" rx="2.5"/><path d="M8 10.5V7.5a4 4 0 0 1 8 0v3"/>`,
    unlock: `<rect x="4.5" y="10.5" width="15" height="10" rx="2.5"/><path d="M8 10.5V7.5a4 4 0 0 1 7.6-1.7"/>`,
    eyeoff: `<path d="M3 3l18 18M10.6 6.1A9.8 9.8 0 0 1 12 6c5 0 8.5 4.2 9.5 6-.4.7-1.2 1.9-2.4 3.1M6.6 7.6C4.5 9 3.1 11 2.5 12c1 1.8 4.5 6 9.5 6 1.6 0 3-.4 4.2-1"/><path d="M9.9 10a3 3 0 0 0 4.1 4.1"/>`,
    eye: `<path d="M2.5 12C3.5 10.2 7 6 12 6s8.5 4.2 9.5 6c-1 1.8-4.5 6-9.5 6s-8.5-4.2-9.5-6Z"/><circle cx="12" cy="12" r="3"/>`,
    ghost: `<path d="M5 20V10a7 7 0 0 1 14 0v10l-2.3-1.8L14.3 20 12 18.2 9.7 20l-2.4-1.8Z"/><path d="M9.5 10.5h.01M14.5 10.5h.01"/>`,
    plus: `<path d="M12 5v14M5 12h14"/>`,
    search: `<circle cx="11" cy="11" r="6.5"/><path d="m20 20-4.2-4.2"/>`,
    play: `<path d="M7 4.8v14.4a1 1 0 0 0 1.5.9l11.3-7.2a1 1 0 0 0 0-1.7L8.5 3.9A1 1 0 0 0 7 4.8Z" fill="currentColor"/>`,
    chevron: `<path d="m6 9 6 6 6-6"/>`,
    chevronRight: `<path d="m9 6 6 6-6 6"/>`,
    more: `<circle cx="5" cy="12" r="1.3" fill="currentColor"/><circle cx="12" cy="12" r="1.3" fill="currentColor"/><circle cx="19" cy="12" r="1.3" fill="currentColor"/>`,
    star: `<path d="m12 3.5 2.6 5.4 5.9.8-4.3 4.1 1 5.8L12 16.9l-5.2 2.7 1-5.8-4.3-4.1 5.9-.8Z"/>`,
    starFill: `<path d="m12 3.5 2.6 5.4 5.9.8-4.3 4.1 1 5.8L12 16.9l-5.2 2.7 1-5.8-4.3-4.1 5.9-.8Z" fill="currentColor"/>`,
    edit: `<path d="M4 20h4L19 9a2.8 2.8 0 0 0-4-4L4 16Z"/><path d="m13.5 6.5 4 4"/>`,
    trash: `<path d="M4 7h16M10 11v6M14 11v6M5.5 7l1 12a2 2 0 0 0 2 1.8h7a2 2 0 0 0 2-1.8l1-12M9 7V4.5h6V7"/>`,
    refresh: `<path d="M20 11a8 8 0 0 0-14.7-4.3L4 8.5M4 4v4.5h4.5M4 13a8 8 0 0 0 14.7 4.3L20 15.5M20 20v-4.5h-4.5"/>`,
    folder: `<path d="M3.5 7.5A2 2 0 0 1 5.5 5.5h3.8l2 2.2h7.2a2 2 0 0 1 2 2v7.8a2 2 0 0 1-2 2h-13a2 2 0 0 1-2-2Z"/>`,
    x: `<path d="M6 6l12 12M18 6 6 18"/>`,
    min: `<path d="M6 12h12"/>`,
    max: `<rect x="6" y="6" width="12" height="12" rx="1.5"/>`,
    restore: `<rect x="5" y="8.5" width="10.5" height="10.5" rx="1.5"/><path d="M8.5 5h9a1.5 1.5 0 0 1 1.5 1.5v9"/>`,
    check: `<path d="m5 12.5 4.5 4.5L19 7.5"/>`,
    copy: `<rect x="8.5" y="8.5" width="11.5" height="11.5" rx="2"/><path d="M15.5 8.5V6a2 2 0 0 0-2-2H6a2 2 0 0 0-2 2v7.5a2 2 0 0 0 2 2h2.5"/>`,
    download: `<path d="M12 4v11M7 10.5l5 5 5-5M4.5 19.5h15"/>`,
    upload: `<path d="M12 15.5v-11M7 9l5-5 5 5M4.5 19.5h15"/>`,
    alert: `<path d="M12 4 2.8 19.5h18.4Z"/><path d="M12 10v4.5M12 17.2h.01"/>`,
    shield: `<path d="M12 3 4.5 6v5.5c0 4.6 3.1 8 7.5 9.5 4.4-1.5 7.5-4.9 7.5-9.5V6Z"/><path d="m9 12 2.2 2.2L15.5 10"/>`,
    info: `<circle cx="12" cy="12" r="8.5"/><path d="M12 11v5M12 8h.01"/>`,
    warning: `<path d="M12 4 2.8 19.5h18.4Z"/><path d="M12 10v4.5M12 17h.01"/>`,
    power: `<path d="M12 3.5v8M7.2 6.5a7 7 0 1 0 9.6 0"/>`,
    sparkle: `<path d="M12 3c.6 4.2 2.8 6.4 7 7-4.2.6-6.4 2.8-7 7-.6-4.2-2.8-6.4-7-7 4.2-.6 6.4-2.8 7-7Z"/><path d="M19 16c.3 1.6 1 2.3 2.5 2.5-1.5.3-2.2 1-2.5 2.5-.3-1.5-1-2.2-2.5-2.5 1.5-.2 2.2-.9 2.5-2.5Z"/>`,
    mouse: `<rect x="6.5" y="3" width="11" height="18" rx="5.5"/><path d="M12 7v3"/>`,
    link: `<path d="M10 14a4 4 0 0 0 5.7 0l3-3a4 4 0 0 0-5.7-5.7l-1 1M14 10a4 4 0 0 0-5.7 0l-3 3a4 4 0 0 0 5.7 5.7l1-1"/>`,
    login: `<path d="M14 4h4a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2h-4M10 16.5 14.5 12 10 7.5M14.5 12H3.5"/>`,
    signal: `<path d="M5 12.5a10 10 0 0 1 14 0M8 15.5a6 6 0 0 1 8 0M11.5 18.5a1 1 0 0 1 1 0"/>`,
    calendar: `<rect x="3.5" y="5" width="17" height="15.5" rx="2.5"/><path d="M3.5 10h17M8 3v4M16 3v4"/>`,
    external: `<path d="M14 4h6v6M20 4l-9 9M18 14v4.5a1.5 1.5 0 0 1-1.5 1.5h-11A1.5 1.5 0 0 1 4 18.5v-11A1.5 1.5 0 0 1 5.5 6H10"/>`,
    grip: `<circle cx="9" cy="6" r="1.2" fill="currentColor"/><circle cx="15" cy="6" r="1.2" fill="currentColor"/><circle cx="9" cy="12" r="1.2" fill="currentColor"/><circle cx="15" cy="12" r="1.2" fill="currentColor"/><circle cx="9" cy="18" r="1.2" fill="currentColor"/><circle cx="15" cy="18" r="1.2" fill="currentColor"/>`,
    image: `<rect x="3.5" y="4.5" width="17" height="15" rx="2.5"/><circle cx="9" cy="10" r="1.8"/><path d="m20.5 16-5-5-9 8.5"/>`,
    target: `<circle cx="12" cy="12" r="8.5"/><circle cx="12" cy="12" r="4.5"/><circle cx="12" cy="12" r="1" fill="currentColor"/>`,
    flame: `<path d="M12 21c-4 0-6.5-2.6-6.5-6.2 0-3.9 3.4-5.6 3.9-9.8 2.6 1.5 4 3.8 4.1 6.3 1-.6 1.7-1.6 1.9-2.8 1.9 1.6 3.1 3.8 3.1 6.3 0 3.6-2.5 6.2-6.5 6.2Z"/>`,
  };
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width={stroke}
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
  style="flex:none"
>
  {@html P[name] ?? ""}
</svg>
