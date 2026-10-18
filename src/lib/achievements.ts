// Succès inter-comptes propres à l'app. Ils rapportent des « éclats » qui débloquent des thèmes.
// Aucun achat en argent réel : les éclats se gagnent uniquement ici, et sont recalculés à partir des données.
import type { Account, ConfigProfile, Folder } from "./types";

export interface Achievement {
  id: string;
  icon: string;
  reward: number;
  progress: number;
  goal: number;
}

export function computeAchievements(accounts: Account[], folders: Folder[], profiles: ConfigProfile[]): Achievement[] {
  const total = (g: "lol" | "tft" | "val") => accounts.reduce((s, a) => s + (a.playtime?.[g] ?? 0), 0);
  const launches = (g: string) => accounts.reduce((s, a) => s + (a.playtime?.launches?.[g] ?? 0), 0);
  const ranked = accounts.filter((a) => a.ranks.lolSolo?.tier || a.ranks.tft?.tier || a.ranks.val?.tier).length;
  const games = ["lol", "tft", "val"].filter((g) => launches(g) > 0).length;
  const twoFa = accounts.length > 0 ? accounts.filter((a) => a.twoFa).length : 0;
  const allHours = (total("lol") + total("tft") + total("val")) / 3600;

  const list: Achievement[] = [
    { id: "first_account", icon: "✦", reward: 25, progress: Math.min(accounts.length, 1), goal: 1 },
    { id: "five_accounts", icon: "✧", reward: 50, progress: Math.min(accounts.length, 5), goal: 5 },
    { id: "organized", icon: "▤", reward: 40, progress: Math.min(folders.length, 3), goal: 3 },
    { id: "first_launch", icon: "▶", reward: 25, progress: Math.min(launches("lol") + launches("tft") + launches("val"), 1), goal: 1 },
    { id: "polyvalent", icon: "◆", reward: 75, progress: games, goal: 3 },
    { id: "ten_hours", icon: "⌛", reward: 60, progress: Math.min(Math.floor(allHours), 10), goal: 10 },
    { id: "hundred_hours", icon: "⏳", reward: 150, progress: Math.min(Math.floor(allHours), 100), goal: 100 },
    { id: "ranked_three", icon: "▲", reward: 80, progress: Math.min(ranked, 3), goal: 3 },
    { id: "secure", icon: "🛡", reward: 100, progress: accounts.length ? twoFa : 0, goal: Math.max(accounts.length, 1) },
    { id: "keybinder", icon: "⌨", reward: 50, progress: Math.min(profiles.length, 1), goal: 1 },
    { id: "switcher", icon: "⇄", reward: 60, progress: Math.min(accounts.filter((a) => a.lastUsedAt).length, 3), goal: 3 },
  ];
  return list;
}

export function shards(list: Achievement[]): number {
  return list.filter((a) => a.progress >= a.goal).reduce((s, a) => s + a.reward, 0);
}
