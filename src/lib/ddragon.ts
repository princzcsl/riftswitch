// Données publiques Data Dragon (identiques pour tous les joueurs, aucun compte requis).
// Mises en cache dans le stockage local pour limiter le réseau.

const CDN = "https://ddragon.leagueoflegends.com";

function cacheGet<T>(key: string, maxAgeMs: number): T | null {
  try {
    const raw = localStorage.getItem(key);
    if (!raw) return null;
    const { at, data } = JSON.parse(raw);
    return Date.now() - at < maxAgeMs ? (data as T) : null;
  } catch {
    return null;
  }
}

function cacheSet(key: string, data: unknown) {
  try {
    localStorage.setItem(key, JSON.stringify({ at: Date.now(), data }));
  } catch {
    /* stockage plein ou indisponible */
  }
}

let versionPromise: Promise<string> | null = null;

export function latestVersion(): Promise<string> {
  if (versionPromise) return versionPromise;
  versionPromise = (async () => {
    const cached = cacheGet<string>("dd.version", 6 * 3600_000);
    if (cached) return cached;
    try {
      const list: string[] = await (await fetch(`${CDN}/api/versions.json`)).json();
      cacheSet("dd.version", list[0]);
      return list[0];
    } catch {
      return cacheGet<string>("dd.version", Infinity) ?? "15.20.1";
    }
  })();
  return versionPromise;
}

export function allVersions(): Promise<string[]> {
  return fetch(`${CDN}/api/versions.json`)
    .then((r) => r.json())
    .catch(() => []);
}

export interface ChampionLite {
  id: string;
  name: string;
  tags: string[];
}

export async function champions(lang = "en_US"): Promise<ChampionLite[]> {
  const v = await latestVersion();
  const key = `dd.champs.${v}.${lang}`;
  const cached = cacheGet<ChampionLite[]>(key, 7 * 86400_000);
  if (cached) return cached;
  try {
    const json = await (await fetch(`${CDN}/cdn/${v}/data/${lang}/champion.json`)).json();
    const list: ChampionLite[] = Object.values(json.data as Record<string, { id: string; name: string; tags: string[] }>)
      .map((c) => ({ id: c.id, name: c.name, tags: c.tags }))
      .sort((a, b) => a.name.localeCompare(b.name));
    cacheSet(key, list);
    return list;
  } catch {
    return [];
  }
}

export function championIcon(version: string, id: string): string {
  return `${CDN}/cdn/${version}/img/champion/${id}.png`;
}

export function profileIcon(version: string, id: number): string {
  return `${CDN}/cdn/${version}/img/profileicon/${id}.png`;
}

export interface RuneStyle {
  id: number;
  name: string;
  icon: string;
  slots: { runes: { id: number; name: string; icon: string }[] }[];
}

export async function runeStyles(lang = "en_US"): Promise<RuneStyle[]> {
  const v = await latestVersion();
  const key = `dd.runes.${v}.${lang}`;
  const cached = cacheGet<RuneStyle[]>(key, 7 * 86400_000);
  if (cached) return cached;
  try {
    const list: RuneStyle[] = await (await fetch(`${CDN}/cdn/${v}/data/${lang}/runesReforged.json`)).json();
    cacheSet(key, list);
    return list;
  } catch {
    return [];
  }
}

export function runeIcon(path: string): string {
  return `${CDN}/cdn/img/${path}`;
}

// ── Valorant (données publiques valorant-api.com, aucune clé) ───────────────

export interface Agent {
  uuid: string;
  name: string;
  icon: string;
}

export async function valorantAgents(): Promise<Agent[]> {
  const cached = cacheGet<Agent[]>("val.agents", 7 * 86400_000);
  if (cached) return cached;
  try {
    const json = await (await fetch("https://valorant-api.com/v1/agents?isPlayableCharacter=true")).json();
    const list: Agent[] = (json.data as { uuid: string; displayName: string; displayIcon: string }[])
      .filter((a) => a.displayIcon)
      .map((a) => ({ uuid: a.uuid, name: a.displayName, icon: a.displayIcon }))
      .sort((a, b) => a.name.localeCompare(b.name));
    cacheSet("val.agents", list);
    return list;
  } catch {
    return [];
  }
}

/** Correspondance langue de l'interface → langue Data Dragon. */
export function ddLang(uiLang: string): string {
  return (
    { fr: "fr_FR", es: "es_ES", it: "it_IT", de: "de_DE", "pt-BR": "pt_BR", pl: "pl_PL", tr: "tr_TR", ru: "ru_RU", ko: "ko_KR", ja: "ja_JP", "zh-TW": "zh_TW", vi: "vi_VN", ar: "ar_AE" } as Record<
      string,
      string
    >
  )[uiLang] ?? "en_US";
}
