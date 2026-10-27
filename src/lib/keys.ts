// Conversion des touches pressées vers les formats des jeux.
//
// League of Legends : positions physiques nommées en QWERTY (scancodes) → `[q]`, `[Shift][q]`, `[Button 4]`.
//   Sur un clavier AZERTY, la touche « A » est stockée `[q]`.
// VALORANT : noms de touches Unreal selon le caractère produit (`Ampersand`, `E_AccentAigu`, `LeftShift`…).

export const LOL_UNBOUND = "[<Unbound>]";

const LOL_CODE: Record<string, string> = {
  Space: "Space",
  Escape: "Esc",
  Tab: "Tab",
  Enter: "Return",
  Backspace: "Back",
  Backquote: "`",
  Minus: "-",
  Equal: "=",
  BracketLeft: "[",
  BracketRight: "]",
  Backslash: "\\",
  Semicolon: ";",
  Quote: "'",
  Comma: ",",
  Period: ".",
  Slash: "/",
  IntlBackslash: "<",
  ArrowUp: "Up Arrow",
  ArrowDown: "Down Arrow",
  ArrowLeft: "Left Arrow",
  ArrowRight: "Right Arrow",
  Insert: "Insert",
  Delete: "Delete",
  Home: "Home",
  End: "End",
  PageUp: "Page Up",
  PageDown: "Page Down",
  CapsLock: "Caps Lock",
  NumpadAdd: "Num +",
  NumpadSubtract: "Num -",
  NumpadMultiply: "Num *",
  NumpadDivide: "Num /",
  NumpadDecimal: "Num .",
  NumpadEnter: "Num Enter",
};

/** Nom LoL d'une touche à partir de son code physique (null si non prise en charge / modificateur seul). */
export function lolKeyName(code: string): string | null {
  if (/^Key[A-Z]$/.test(code)) return code.slice(3).toLowerCase();
  if (/^Digit[0-9]$/.test(code)) return code.slice(5);
  if (/^F([1-9]|1[0-2])$/.test(code)) return code;
  if (/^Numpad[0-9]$/.test(code)) return `Num ${code.slice(6)}`;
  if (["ShiftLeft", "ShiftRight", "ControlLeft", "ControlRight", "AltLeft", "AltRight", "MetaLeft", "MetaRight"].includes(code)) return null;
  return LOL_CODE[code] ?? null;
}

export function lolMouseName(button: number): string | null {
  // 0 gauche, 1 milieu, 2 droit, 3/4 latéraux. LoL : Button 1 = gauche, 2 = droit, 3 = milieu, 4/5 = latéraux.
  return ({ 0: "Button 1", 2: "Button 2", 1: "Button 3", 3: "Button 4", 4: "Button 5" } as Record<number, string>)[button] ?? null;
}

export function lolCombo(key: string, mods: { shift: boolean; ctrl: boolean; alt: boolean }): string {
  return `${mods.ctrl ? "[Ctrl]" : ""}${mods.alt ? "[Alt]" : ""}${mods.shift ? "[Shift]" : ""}[${key}]`;
}

/** Découpe une valeur input.ini en liaisons : `[Shift][q],[Ctrl][q]` → 2 liaisons. */
export function splitLolValue(value: string): string[] {
  if (!value) return [];
  const out: string[] = [];
  let depth = 0;
  let cur = "";
  for (const ch of value) {
    if (ch === "[") depth++;
    if (ch === "]") depth--;
    if (ch === "," && depth === 0) {
      out.push(cur.trim());
      cur = "";
    } else cur += ch;
  }
  if (cur.trim()) out.push(cur.trim());
  return out;
}

export function joinLolValue(bindings: string[]): string {
  const clean = bindings.map((b) => b.trim()).filter((b) => b && b !== LOL_UNBOUND);
  return clean.length ? clean.join(",") : LOL_UNBOUND;
}

let layoutMap: Map<string, string> | null = null;

/** Disposition clavier réelle (Chromium) pour afficher `[q]` comme « A » sur AZERTY. */
export async function loadLayout() {
  try {
    const kb = (navigator as unknown as { keyboard?: { getLayoutMap: () => Promise<Map<string, string>> } }).keyboard;
    if (kb) layoutMap = await kb.getLayoutMap();
  } catch {
    layoutMap = null;
  }
}

/** Affichage lisible d'une liaison LoL selon la disposition de l'utilisateur. */
export function prettyLol(binding: string): string {
  if (!binding || binding === LOL_UNBOUND) return "";
  const parts = binding.match(/\[[^\]]*\]/g) ?? [binding];
  return parts
    .map((p) => {
      const k = p.slice(1, -1);
      if (k.length === 1 && /[a-z]/.test(k) && layoutMap) return (layoutMap.get(`Key${k.toUpperCase()}`) ?? k).toUpperCase();
      if (k.length === 1 && /[0-9]/.test(k) && layoutMap) {
        const c = layoutMap.get(`Digit${k}`);
        return c && c !== k ? `${c} (${k})` : k;
      }
      if (k.startsWith("Button ")) return `🖱${({ "1": "L", "2": "R", "3": "M" } as Record<string, string>)[k.slice(7)] ?? k.slice(7)}`;
      return k.length === 1 ? k.toUpperCase() : k;
    })
    .join(" + ");
}

// ── VALORANT ────────────────────────────────────────────────────────────────

export interface ValKey {
  key: string;
  shift: boolean;
  ctrl: boolean;
  alt: boolean;
}

const VAL_CHAR: Record<string, string> = {
  "&": "Ampersand",
  é: "E_AccentAigu",
  '"': "Quote",
  "'": "Apostrophe",
  "(": "LeftParantheses",
  ")": "RightParantheses",
  "-": "Hyphen",
  è: "E_AccentGrave",
  _: "Underscore",
  ç: "C_Cedille",
  à: "A_AccentGrave",
  "=": "Equals",
  ",": "Comma",
  ";": "Semicolon",
  ":": "Colon",
  "!": "Exclamation",
  "*": "Asterix",
  $: "Dollar",
  "^": "Caret",
  ù: "U_AccentGrave",
  "²": "Section",
  "<": "Less",
  ".": "Period",
  "/": "Slash",
  "`": "Tilde",
  "[": "LeftBracket",
  "]": "RightBracket",
  "\\": "Backslash",
};

const VAL_CODE: Record<string, string> = {
  Space: "SpaceBar",
  Escape: "Escape",
  Tab: "Tab",
  Enter: "Enter",
  Backspace: "BackSpace",
  ShiftLeft: "LeftShift",
  ShiftRight: "RightShift",
  ControlLeft: "LeftControl",
  ControlRight: "RightControl",
  AltLeft: "LeftAlt",
  AltRight: "RightAlt",
  CapsLock: "CapsLock",
  ArrowUp: "Up",
  ArrowDown: "Down",
  ArrowLeft: "Left",
  ArrowRight: "Right",
  Insert: "Insert",
  Delete: "Delete",
  Home: "Home",
  End: "End",
  PageUp: "PageUp",
  PageDown: "PageDown",
};

const DIGIT_NAMES = ["Zero", "One", "Two", "Three", "Four", "Five", "Six", "Seven", "Eight", "Nine"];

export function valKeyName(e: KeyboardEvent): string | null {
  if (VAL_CODE[e.code]) return VAL_CODE[e.code];
  if (/^F([1-9]|1[0-2])$/.test(e.code)) return e.code;
  if (/^Numpad[0-9]$/.test(e.code)) return `NumPad${DIGIT_NAMES[Number(e.code.slice(6))]}`;
  const k = e.key;
  if (k.length === 1) {
    if (/[a-z]/i.test(k)) return k.toUpperCase();
    if (/[0-9]/.test(k)) return DIGIT_NAMES[Number(k)];
    if (VAL_CHAR[k.toLowerCase()]) return VAL_CHAR[k.toLowerCase()];
  }
  return null;
}

export function valMouseName(button: number): string | null {
  return (
    ({ 0: "LeftMouseButton", 1: "MiddleMouseButton", 2: "RightMouseButton", 3: "ThumbMouseButton", 4: "ThumbMouseButton2" } as Record<number, string>)[
      button
    ] ?? null
  );
}

const VAL_PRETTY: Record<string, string> = Object.fromEntries([
  ...Object.entries(VAL_CHAR).map(([c, n]) => [n, c]),
  ["SpaceBar", "Space"],
  ["LeftShift", "Shift"],
  ["LeftControl", "Ctrl"],
  ["LeftAlt", "Alt"],
  ["LeftMouseButton", "🖱L"],
  ["RightMouseButton", "🖱R"],
  ["MiddleMouseButton", "🖱M"],
  ["ThumbMouseButton", "🖱4"],
  ["ThumbMouseButton2", "🖱5"],
  ["MouseScrollUp", "🖱↑"],
  ["MouseScrollDown", "🖱↓"],
  ...DIGIT_NAMES.map((n, i) => [n, String(i)]),
]);

export function prettyVal(key: string): string {
  if (!key || key === "None") return "";
  return VAL_PRETTY[key] ?? key;
}
