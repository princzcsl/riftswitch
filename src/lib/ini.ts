// INI des profils de config : lecture / écriture (même format que le cœur Rust), libellés lisibles
// et tri des clés (propres au PC, son VALORANT). Garder les listes alignées avec `configs/mod.rs`.
import type { IniDoc } from "./types";

export function parseIni(text: string): IniDoc {
  const doc: IniDoc = { sections: [] };
  for (const raw of text.split(/\r?\n/)) {
    const line = raw.trim();
    if (!line || line.startsWith(";") || line.startsWith("#")) continue;
    if (line.startsWith("[") && line.endsWith("]") && !line.includes("=")) {
      doc.sections.push({ name: line.slice(1, -1), entries: [] });
      continue;
    }
    const i = line.indexOf("=");
    if (i < 0) continue;
    if (!doc.sections.length) doc.sections.push({ name: "", entries: [] });
    doc.sections[doc.sections.length - 1].entries.push([line.slice(0, i).trim(), line.slice(i + 1).trim()]);
  }
  return doc;
}

export function iniText(doc: IniDoc): string {
  return doc.sections
    .map((s) => (s.name ? `[${s.name}]\r\n` : "") + s.entries.map(([k, v]) => `${k}=${v}\r\n`).join("") + "\r\n")
    .join("");
}

const bare = (key: string) => key.split("::").pop() ?? key;

/** Clé propre à un PC ou à une version du jeu : ni affichée ni copiée. */
export function isVolatile(key: string): boolean {
  const k = bare(key);
  return (
    k.endsWith("Version") ||
    ["DeviceID", "MonitorIndex", "Benchmark", "LastConfirmed", "LastUserConfirmed", "LastRecommended", "WindowPos", "PreferredMonitor", "LastSeen", "Tutorial"].some((p) => k.includes(p)) ||
    ["HasSeen", "HasAccepted", "HasEver", "LastAccepted", "ShootingRange", "ContextAware"].some((p) => k.startsWith(p))
  );
}

const LOL_GENERAL_GRAPHICS = ["Width", "Height", "WindowMode", "WaitForVerticalSync", "ShowGodray", "HideEyeCandy"];
const LOL_GENERAL_SOUND = ["EnableAudio", "ThemeMusic", "EnableCustomAnnouncer", "EnableCosmeticArenaAudioTFT", "EnableAllCosmeticArenaAudioTFT"];

/** Catégorie d'une clé de game.cfg (null : jamais copiée). */
export function lolCat(section: string, key: string): string | null {
  if (section === "Volume" || section === "Voice") return "sound";
  if (section === "Performance") return "graphics";
  if (section === "General" && LOL_GENERAL_GRAPHICS.includes(key)) return "graphics";
  if (section === "General" && LOL_GENERAL_SOUND.includes(key)) return "sound";
  if (["Mobile", "GameModes", "Replay"].includes(section)) return null;
  return "interface";
}

/** Catégorie d'une clé de RiotUserSettings (VALORANT). */
const VAL_GRAPHICS = ["Quality", "Framerate", "Vignette", "Anisotropic", "AntiAliasing", "PlayerPerf", "Bloom", "Distortion", "Shadow", "Clarity", "Sharpening", "VSync"];

export function valCat(key: string): string {
  const k = bare(key);
  if (k.includes("Crosshair")) return "crosshair";
  if (VAL_GRAPHICS.some((p) => k.includes(p))) return "graphics";
  return isValSound(key) ? "sound" : "interface";
}

export function isValSound(key: string): boolean {
  const k = bare(key);
  return ["Volume", "Music", "Sound", "Audio", "Voice", "Voip", "PushToTalk", "HRTF", "Speaker", "Microphone", "Mic"].some((p) => k.includes(p));
}

/** « EAresFloatSettingName::MouseSensitivity » → « Mouse Sensitivity », « bUseVSync » → « Use V Sync ». */
export function humanize(key: string): string {
  return bare(key)
    .split(".")
    .pop()!
    .replace(/^b(?=[A-Z])/, "")
    .replace(/_/g, " ")
    .replace(/([a-z0-9])([A-Z])/g, "$1 $2")
    .replace(/([A-Z]+)([A-Z][a-z])/g, "$1 $2")
    .trim();
}

/** Groupe d'affichage d'une clé VALORANT (RiotUserSettings). */
export function valGroup(key: string): string {
  const k = bare(key);
  if (/Crosshair/.test(k)) return "crosshair";
  if (/Minimap/.test(k)) return "minimap";
  if (/Mouse|Sensitivity|Aim|Zoom|Invert/.test(k)) return "mouse";
  if (isValSound(k)) return "sound";
  return "other";
}
