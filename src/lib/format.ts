import type { Account, Rank } from "./types";
import { t } from "./i18n.svelte";

export const REGIONS = ["EUW", "EUNE", "NA", "KR", "JP", "BR", "LAN", "LAS", "OCE", "TR", "RU", "ME", "SEA", "TW", "VN", "PBE"];

/** Langues proposées par les jeux Riot (code Riot → nom natif). */
export const GAME_LOCALES: [string, string][] = [
  ["en_US", "English (US)"],
  ["en_GB", "English (UK)"],
  ["fr_FR", "Français"],
  ["de_DE", "Deutsch"],
  ["es_ES", "Español (España)"],
  ["es_MX", "Español (Latinoamérica)"],
  ["it_IT", "Italiano"],
  ["pt_BR", "Português (Brasil)"],
  ["pl_PL", "Polski"],
  ["tr_TR", "Türkçe"],
  ["ru_RU", "Русский"],
  ["cs_CZ", "Čeština"],
  ["el_GR", "Ελληνικά"],
  ["hu_HU", "Magyar"],
  ["ro_RO", "Română"],
  ["ko_KR", "한국어"],
  ["ja_JP", "日本語"],
  ["zh_TW", "繁體中文"],
  ["zh_CN", "简体中文"],
  ["vi_VN", "Tiếng Việt"],
  ["th_TH", "ไทย"],
  ["id_ID", "Bahasa Indonesia"],
  ["ar_AE", "العربية"],
];

export const TIERS = ["IRON", "BRONZE", "SILVER", "GOLD", "PLATINUM", "EMERALD", "DIAMOND", "MASTER", "GRANDMASTER", "CHALLENGER"];
export const VAL_TIERS = ["IRON", "BRONZE", "SILVER", "GOLD", "PLATINUM", "DIAMOND", "ASCENDANT", "IMMORTAL", "RADIANT"];

export const TIER_COLORS: Record<string, string> = {
  IRON: "#7d6d66",
  BRONZE: "#b0703f",
  SILVER: "#a7b2c0",
  GOLD: "#e8b94a",
  PLATINUM: "#4fc1b0",
  EMERALD: "#2fd07a",
  DIAMOND: "#7b95ff",
  MASTER: "#b866ff",
  GRANDMASTER: "#ff4f63",
  CHALLENGER: "#f7d87c",
  ASCENDANT: "#2ecc8f",
  IMMORTAL: "#e0405a",
  RADIANT: "#ffe680",
};

const APEX = ["MASTER", "GRANDMASTER", "CHALLENGER", "IMMORTAL", "RADIANT"];

export function tierLabel(tier: string): string {
  const key = `tiers.${tier.toLowerCase()}`;
  const s = t(key);
  return s === key ? tier.charAt(0) + tier.slice(1).toLowerCase() : s;
}

export function rankShort(r: Rank | null | undefined): string {
  if (!r || !r.tier) return t("ranks.unranked");
  const div = APEX.includes(r.tier) || !r.division ? "" : ` ${r.division}`;
  return `${tierLabel(r.tier)}${div}`;
}

export function rankFull(r: Rank | null | undefined): string {
  if (!r || !r.tier) return t("ranks.unranked");
  return `${rankShort(r)} · ${r.lp} ${t("ranks.lp")}`;
}

/** Score numérique d'un rang (pour trier / comparer). */
export function rankScore(r: Rank | null | undefined, val = false): number {
  if (!r || !r.tier) return -1;
  const list = val ? VAL_TIERS : TIERS;
  const tier = list.indexOf(r.tier);
  const div = { I: 3, II: 2, III: 1, IV: 0, "1": 0, "2": 1, "3": 2 }[r.division] ?? 0;
  return tier * 400 + div * 100 + Math.min(r.lp, 100);
}

export function bestRank(a: Account): { game: string; rank: Rank } | null {
  const c: [string, Rank | null][] = [
    ["lol", a.ranks.lolSolo],
    ["tft", a.ranks.tft],
    ["val", a.ranks.val],
  ];
  let best: { game: string; rank: Rank } | null = null;
  for (const [game, r] of c) {
    if (r && r.tier && (!best || rankScore(r, game === "val") > rankScore(best.rank, best.game === "val"))) best = { game, rank: r };
  }
  return best;
}

export function hours(secs: number): string {
  if (!secs) return "0 h";
  const h = secs / 3600;
  return h < 10 ? `${h.toFixed(1)} h` : `${Math.round(h)} h`;
}

export function relativeTime(ms: number | null | undefined): string {
  if (!ms) return t("time.never");
  const diff = Date.now() - ms;
  const min = Math.round(diff / 60000);
  if (min < 1) return t("time.now");
  if (min < 60) return t("time.minutes", { n: min });
  const h = Math.round(min / 60);
  if (h < 24) return t("time.hours", { n: h });
  const d = Math.round(h / 24);
  if (d < 31) return t("time.days", { n: d });
  return new Date(ms).toLocaleDateString();
}

export function recentPlaytime(a: Account, days = 7): number {
  const today = Math.floor(Date.now() / 86_400_000);
  return Object.entries(a.playtime.recent ?? {})
    .filter(([d]) => today - Number(d) < days)
    .reduce((s, [, v]) => s + v, 0);
}

export function displayRiotId(a: Account, streamer: boolean): string {
  if (streamer) return a.gameName ? `${a.gameName.slice(0, 1)}•••#•••` : "";
  return a.riotId;
}
