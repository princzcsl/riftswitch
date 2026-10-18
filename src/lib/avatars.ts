// Set d'icônes de compte original (aucun asset Riot) : emblèmes et petits personnages chibi.
// Chaque entrée est un fragment SVG statique dans un viewBox 0 0 64 64 ; `currentColor` = couleur du compte.

export interface Glyph {
  id: string;
  svg: string;
}

const face = (extra: string, eyes = "") => `
  ${extra}
  <circle cx="32" cy="36" r="18" fill="#fff" fill-opacity=".95"/>
  ${eyes || `<ellipse cx="25" cy="36" rx="3.4" ry="4.4" fill="#1a1838"/><ellipse cx="39" cy="36" rx="3.4" ry="4.4" fill="#1a1838"/>
  <circle cx="26.2" cy="34.4" r="1.2" fill="#fff"/><circle cx="40.2" cy="34.4" r="1.2" fill="#fff"/>`}
  <ellipse cx="21" cy="42" rx="3" ry="1.8" fill="#ff8fb8" fill-opacity=".7"/><ellipse cx="43" cy="42" rx="3" ry="1.8" fill="#ff8fb8" fill-opacity=".7"/>
  <path d="M29 43 Q32 46 35 43" stroke="#1a1838" stroke-width="1.8" fill="none" stroke-linecap="round"/>`;

export const GLYPHS: Glyph[] = [
  { id: "0", svg: face(`<path d="M18 26 L14 10 L26 20 Z M46 26 L50 10 L38 20 Z" fill="#ffd36b"/>`) },
  { id: "1", svg: face(`<path d="M17 28 L15 12 L27 22 Z M47 28 L49 12 L37 22 Z" fill="currentColor"/><path d="M19 25 L18 16 L24 21 Z M45 25 L46 16 L40 21 Z" fill="#ffc2d8"/>`) },
  { id: "2", svg: face(`<ellipse cx="24" cy="12" rx="4.5" ry="12" fill="#fff" stroke="currentColor" stroke-width="2"/><ellipse cx="40" cy="12" rx="4.5" ry="12" fill="#fff" stroke="currentColor" stroke-width="2"/>`) },
  { id: "3", svg: face(`<path d="M14 30 Q32 6 50 30 Q32 22 14 30 Z" fill="currentColor"/><circle cx="32" cy="14" r="3" fill="#ffd36b"/>`, `<path d="M22 37 Q25 33 28 37" stroke="#1a1838" stroke-width="2" fill="none" stroke-linecap="round"/><path d="M36 37 Q39 33 42 37" stroke="#1a1838" stroke-width="2" fill="none" stroke-linecap="round"/>`) },
  { id: "4", svg: `<path d="M32 8 L38 26 L56 26 L41 37 L47 55 L32 44 L17 55 L23 37 L8 26 L26 26 Z" fill="#fff" fill-opacity=".95"/>` },
  { id: "5", svg: `<path d="M32 6 C40 18 48 24 44 38 C42 48 36 54 32 56 C26 54 18 48 18 38 C18 30 24 26 26 18 C28 24 30 26 32 28 C34 20 32 14 32 6 Z" fill="#fff" fill-opacity=".95"/>` },
  { id: "6", svg: `<path d="M14 50 C14 26 30 12 52 12 C52 34 38 50 14 50 Z" fill="#fff" fill-opacity=".95"/><path d="M14 50 L40 24" stroke="currentColor" stroke-width="3"/>` },
  { id: "7", svg: `<path d="M36 6 L16 36 L30 36 L26 58 L48 26 L34 26 Z" fill="#fff" fill-opacity=".95"/>` },
  { id: "8", svg: `<path d="M42 8 A24 24 0 1 0 56 40 A18 18 0 1 1 42 8 Z" fill="#fff" fill-opacity=".95"/>` },
  { id: "9", svg: `<path d="M32 8 L52 18 L52 34 C52 46 42 54 32 58 C22 54 12 46 12 34 L12 18 Z" fill="#fff" fill-opacity=".95"/><path d="M32 18 L32 48 M20 30 L44 30" stroke="currentColor" stroke-width="3.5"/>` },
  { id: "10", svg: `<path d="M18 22 L32 8 L46 22 L32 56 Z" fill="#fff" fill-opacity=".95"/><path d="M18 22 L46 22 M32 8 L26 22 L32 56 L38 22 Z" stroke="currentColor" stroke-width="1.6" fill="none"/>` },
  { id: "11", svg: `<path d="M12 46 L46 12 M40 12 L52 12 L52 24" stroke="#fff" stroke-width="5" fill="none" stroke-linecap="round"/><path d="M12 46 L22 52 M12 46 L18 36" stroke="#fff" stroke-width="5" stroke-linecap="round"/>` },
  { id: "12", svg: `<path d="M12 44 L16 20 L26 32 L32 14 L38 32 L48 20 L52 44 Z" fill="#fff" fill-opacity=".95"/><rect x="12" y="46" width="40" height="6" rx="2" fill="#fff" fill-opacity=".95"/>` },
  { id: "13", svg: `<circle cx="32" cy="32" r="12" fill="#fff" fill-opacity=".95"/><path d="M32 6 V14 M32 50 V58 M6 32 H14 M50 32 H58 M13 13 L19 19 M45 45 L51 51 M13 51 L19 45 M45 19 L51 13" stroke="#fff" stroke-width="4" stroke-linecap="round"/>` },
  { id: "14", svg: `<ellipse cx="32" cy="32" rx="24" ry="14" fill="#fff" fill-opacity=".95"/><circle cx="32" cy="32" r="8" fill="currentColor"/><circle cx="32" cy="32" r="3.5" fill="#1a1838"/>` },
  { id: "15", svg: `<circle cx="22" cy="20" r="6" fill="#fff"/><circle cx="42" cy="20" r="6" fill="#fff"/><circle cx="13" cy="33" r="5.5" fill="#fff"/><circle cx="51" cy="33" r="5.5" fill="#fff"/><path d="M32 30 C42 30 48 42 44 50 C40 56 24 56 20 50 C16 42 22 30 32 30 Z" fill="#fff"/>` },
];

export const ACCOUNT_COLORS = [
  "#7c5cff",
  "#2fd4ff",
  "#ff5c8a",
  "#ffb547",
  "#3ddc97",
  "#ff7a45",
  "#5c8dff",
  "#c86bff",
  "#00c2a8",
  "#e8c547",
  "#ff4655",
  "#8a8fa8",
];
