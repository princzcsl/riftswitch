// i18n minimaliste : un fichier JSON par langue, clés à points, repli sur l'anglais.
// Ajouter une langue = ajouter un JSON dans src/locales et l'importer ici.
import en from "../locales/en.json";
import fr from "../locales/fr.json";
import es from "../locales/es.json";
import it from "../locales/it.json";
import de from "../locales/de.json";

type Dict = Record<string, string>;

function flatten(obj: Record<string, unknown>, prefix = "", out: Dict = {}): Dict {
  for (const [k, v] of Object.entries(obj)) {
    const key = prefix ? `${prefix}.${k}` : k;
    if (v && typeof v === "object") flatten(v as Record<string, unknown>, key, out);
    else out[key] = String(v);
  }
  return out;
}

const dicts: Record<string, Dict> = {
  en: flatten(en),
  fr: flatten(fr),
  es: flatten(es),
  it: flatten(it),
  de: flatten(de),
};

/** Langues de l'interface. `complete: false` = repli partiel sur l'anglais (structure prête). */
export const LANGUAGES: { code: string; name: string; complete: boolean; rtl?: boolean }[] = [
  { code: "en", name: "English", complete: true },
  { code: "fr", name: "Français", complete: true },
  { code: "es", name: "Español", complete: true },
  { code: "it", name: "Italiano", complete: true },
  { code: "de", name: "Deutsch", complete: true },
  { code: "pt-BR", name: "Português (Brasil)", complete: false },
  { code: "pl", name: "Polski", complete: false },
  { code: "tr", name: "Türkçe", complete: false },
  { code: "ru", name: "Русский", complete: false },
  { code: "ko", name: "한국어", complete: false },
  { code: "ja", name: "日本語", complete: false },
  { code: "zh-TW", name: "繁體中文", complete: false },
  { code: "vi", name: "Tiếng Việt", complete: false },
  { code: "ar", name: "العربية", complete: false, rtl: true },
];

export const i18n = $state({ lang: "en" });

export function detectLanguage(): string {
  const nav = (navigator.language || "en").toLowerCase();
  const exact = LANGUAGES.find((l) => l.code.toLowerCase() === nav);
  if (exact) return exact.code;
  const base = nav.split("-")[0];
  return LANGUAGES.find((l) => l.code.toLowerCase().split("-")[0] === base)?.code ?? "en";
}

export function setLanguage(code: string) {
  i18n.lang = code || "en";
  const meta = LANGUAGES.find((l) => l.code === i18n.lang);
  document.documentElement.lang = i18n.lang;
  document.documentElement.dir = meta?.rtl ? "rtl" : "ltr";
}

/** Traduit une clé ; `{name}` est remplacé par params.name. */
export function t(key: string, params?: Record<string, string | number>): string {
  const lang = i18n.lang;
  const dict = dicts[lang] ?? dicts[lang.split("-")[0]];
  let s = dict?.[key] ?? dicts.en[key] ?? key;
  if (params) for (const [k, v] of Object.entries(params)) s = s.replaceAll(`{${k}}`, String(v));
  return s;
}

/** Message lisible pour une erreur du cœur Rust (code stable → texte traduit). */
export function errorText(e: unknown): string {
  const err = e as { code?: string; detail?: string };
  if (err && typeof err === "object" && err.code) {
    const key = `errors.${err.code}`;
    const txt = t(key);
    return txt === key ? `${t("errors.generic")} (${err.code}${err.detail ? `: ${err.detail}` : ""})` : txt;
  }
  return String(e);
}
