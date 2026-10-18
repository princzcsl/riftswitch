// Système multi-thème : un thème = un fichier JSON de jetons (couleurs, flou, transparence, rayons).
// Ajouter un thème = déposer un JSON dans src/themes, sans toucher au code.
import { getCurrentWindow, Effect } from "@tauri-apps/api/window";

export interface Theme {
  id: string;
  name: string;
  dark: boolean;
  /** Coût en éclats (monnaie de succès) ; 0 = disponible d'office. */
  cost: number;
  windowEffect: "mica" | "acrylic" | "none";
  tokens: Record<string, string>;
}

const modules = import.meta.glob<{ default: Theme }>("../themes/*.json", { eager: true });
export const THEMES: Theme[] = Object.values(modules)
  .map((m) => m.default)
  .sort((a, b) => a.cost - b.cost || a.name.localeCompare(b.name));

export function getTheme(id: string): Theme {
  return THEMES.find((t) => t.id === id) ?? THEMES.find((t) => t.id === "liquid-glass-dark") ?? THEMES[0];
}

let lastEffect = "";

export async function applyTheme(id: string, performance: boolean) {
  const theme = getTheme(id);
  const root = document.documentElement;
  for (const [k, v] of Object.entries(theme.tokens)) root.style.setProperty(`--${k}`, v);
  root.dataset.theme = theme.id;
  root.dataset.scheme = theme.dark ? "dark" : "light";
  root.classList.toggle("perf", performance);

  // Effet natif de la fenêtre (Mica / Acrylic sous Windows 11). Coupé en mode performance.
  const effect = performance ? "none" : theme.windowEffect;
  if (effect === lastEffect) return;
  lastEffect = effect;
  try {
    const win = getCurrentWindow();
    await win.setTheme(theme.dark ? "dark" : "light");
    if (effect === "none") await win.setEffects({ effects: [] });
    else await win.setEffects({ effects: [effect === "acrylic" ? Effect.Acrylic : Effect.Mica] });
  } catch {
    /* Windows 10 / macOS : l'effet CSS suffit */
  }
}

/** Mode performance automatique : petite config ou sur batterie. */
export async function autoPerformance(): Promise<boolean> {
  const cores = navigator.hardwareConcurrency ?? 8;
  const mem = (navigator as unknown as { deviceMemory?: number }).deviceMemory ?? 8;
  if (cores <= 4 || mem <= 4) return true;
  try {
    const nav = navigator as unknown as { getBattery?: () => Promise<{ charging: boolean }> };
    if (nav.getBattery) {
      const b = await nav.getBattery();
      if (!b.charging) return true;
    }
  } catch {
    /* API batterie indisponible */
  }
  return false;
}
