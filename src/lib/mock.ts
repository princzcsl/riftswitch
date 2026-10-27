// Mode démo (développement uniquement) : simule le cœur Rust dans un navigateur classique,
// avec des comptes fictifs, pour travailler l'interface sans toucher au Riot Client ni au coffre.
// Jamais inclus dans le build de production (import conditionnel dans main.ts).
import { mockIPC, mockWindows } from "@tauri-apps/api/mocks";

const now = Date.now();
const day = 86_400_000;

const rank = (tier: string, division: string, lp: number) => ({ tier, division, lp, wins: 40, losses: 35 });

const accounts = [
  {
    id: "a1",
    label: "Main EUW",
    gameName: "Prince",
    tagLine: "EUW",
    puuid: "p1",
    maskedEmail: "pr***@gm***.com",
    region: "EUW",
    color: "#7c5cff",
    icon: "champ:Ahri",
    tags: ["main"],
    folderId: "f1",
    favorite: true,
    createdAt: now - 90 * day,
    lastUsedAt: now - 3600_000,
    lastGame: "lol",
    langs: { lol: "fr_FR", tft: null, val: "ja_JP" },
    twoFa: true,
    notes: "",
    championPool: ["Ahri", "Syndra"],
    agentPool: ["Jett"],
    roles: ["mid"],
    aliases: [{ riotId: "PrinceOld#EUW", seenAt: now - 60 * day }],
    ranks: { lolSolo: rank("DIAMOND", "II", 64), lolFlex: null, tft: rank("PLATINUM", "I", 20), val: rank("ASCENDANT", "2", 41), updatedAt: now, manual: false },
    decay: { daysRemaining: 6, bankedGames: 2, updatedAt: now },
    health: { penalties: [], honorLevel: 4, updatedAt: now },
    playtime: { lol: 86_000, tft: 12_000, val: 30_000, recent: { [Math.floor(now / day)]: 7200 }, launches: { lol: 12, val: 3 }, launcher: { lol: 21_000, tft: 3_600, val: 9_000 } },
    profiles: { lol: "pr1", val: null },
    sessionCapturedAt: now - 2 * day,
    hasSession: true,
    hasPassword: false,
    hasUsername: false,
    profileIconId: 5367,
    valShard: "eu",
    level: 412,
    riotId: "Prince#EUW",
  },
  {
    id: "a2",
    label: "Smurf jungle",
    gameName: "NightOwl",
    tagLine: "1337",
    puuid: "p2",
    maskedEmail: "ow***@pr***.me",
    region: "EUW",
    color: "#2fd4ff",
    icon: "glyph:3",
    tags: ["smurf", "jungle"],
    folderId: "f2",
    favorite: true,
    createdAt: now - 40 * day,
    lastUsedAt: now - 2 * day,
    lastGame: "lol",
    langs: { lol: "en_US", tft: null, val: null },
    twoFa: false,
    notes: "",
    championPool: ["Lee Sin"],
    agentPool: [],
    roles: ["jungle"],
    aliases: [],
    ranks: { lolSolo: rank("EMERALD", "IV", 12), lolFlex: null, tft: null, val: null, updatedAt: now, manual: false },
    decay: { daysRemaining: null, bankedGames: null, updatedAt: now },
    health: { penalties: [], honorLevel: 2, updatedAt: now },
    playtime: { lol: 40_000, tft: 0, val: 0, recent: {}, launches: { lol: 4 } },
    profiles: { lol: null, val: null },
    sessionCapturedAt: now - 5 * day,
    hasSession: true,
    hasPassword: true,
    hasUsername: true,
    profileIconId: null,
    valShard: null,
    level: 87,
    riotId: "NightOwl#1337",
  },
  {
    id: "a3",
    label: "Duo VALORANT",
    gameName: "Kairo",
    tagLine: "VAL",
    puuid: "p3",
    maskedEmail: "",
    region: "EUW",
    color: "#ff4655",
    icon: "glyph:7",
    tags: ["duo"],
    folderId: null,
    favorite: false,
    createdAt: now - 10 * day,
    lastUsedAt: now - 6 * day,
    lastGame: "val",
    langs: { lol: null, tft: null, val: null },
    twoFa: true,
    notes: "",
    championPool: [],
    agentPool: ["Omen", "Sova"],
    roles: [],
    aliases: [],
    ranks: { lolSolo: null, lolFlex: null, tft: null, val: rank("GOLD", "3", 55), updatedAt: now, manual: true },
    decay: { daysRemaining: null, bankedGames: null, updatedAt: null },
    health: { penalties: [], honorLevel: null, updatedAt: null },
    playtime: { lol: 0, tft: 0, val: 20_000, recent: {}, launches: { val: 5 } },
    profiles: { lol: null, val: null },
    sessionCapturedAt: null,
    hasSession: false,
    hasPassword: false,
    hasUsername: false,
    profileIconId: null,
    valShard: null,
    level: null,
    riotId: "Kairo#VAL",
  },
  {
    id: "a4",
    label: "TFT chill",
    gameName: "Pengu",
    tagLine: "TFT",
    puuid: "p4",
    maskedEmail: "pe***@ou***.fr",
    region: "EUNE",
    color: "#ffb547",
    icon: "glyph:15",
    tags: [],
    folderId: "f2",
    favorite: false,
    createdAt: now - 20 * day,
    lastUsedAt: now - 12 * day,
    lastGame: "tft",
    langs: { lol: null, tft: "ko_KR", val: null },
    twoFa: true,
    notes: "",
    championPool: [],
    agentPool: [],
    roles: [],
    aliases: [],
    ranks: { lolSolo: rank("GOLD", "I", 88), lolFlex: null, tft: rank("MASTER", "I", 210), val: null, updatedAt: now, manual: false },
    decay: { daysRemaining: null, bankedGames: null, updatedAt: null },
    health: { penalties: ["ranked_restriction:3"], honorLevel: 1, updatedAt: now },
    playtime: { lol: 4_000, tft: 60_000, val: 0, recent: {}, launches: { tft: 9 } },
    profiles: { lol: null, val: null },
    sessionCapturedAt: now - day,
    hasSession: true,
    hasPassword: false,
    hasUsername: false,
    profileIconId: null,
    valShard: null,
    level: 150,
    riotId: "Pengu#TFT",
  },
];

const folders = [
  { id: "f1", name: "Mains", color: "#7c5cff", order: 0 },
  { id: "f2", name: "Smurfs", color: "#2fd4ff", order: 1 },
];

const input = `[GameEvents]
evtCastSpell1=[q]
evtCastSpell2=[w]
evtCastSpell3=[e]
evtCastSpell4=[r]
evtCastAvatarSpell1=[d]
evtCastAvatarSpell2=[f]
evtUseItem1=[Button 5]
evtUseItem2=[Button 4]
evtUseItem3=[x]
evtUseItem4=[5]
evtUseItem5=[6]
evtUseItem6=[7]
evtUseVisionItem=[4]
evtUseItem7=[b]
evtPlayerAttackMove=[a]
evtPlayerHoldPosition=[h]
evtPlayerStopPosition=[s]
evtCameraLockToggle=[y]
evtCameraSnap=[Space]
evtSelectSelf=[F1]
evtLevelSpell1=[Ctrl][q]
evtLevelSpell2=[Ctrl][w]
evtLevelSpell3=[Ctrl][e]
evtLevelSpell4=[Ctrl][r]
evtSmartCastSpell1=[Shift][q]
evntPlayerPing=[g]
evtChampionOnly=[\`]
evtOpenShop=[p]
evtEmoteDance=[Ctrl][3]
[HUDEvents]
evtHoldShowScoreBoard=[Tab]
evtToggleFPSAndLatency=[Ctrl][f]
[Quickbinds]
evtCastSpell1smart=1
evtCastSpell2smart=1
[ShopEvents]
evtShopFocusSearch=[<Unbound>]`;

function parseIni(text: string) {
  const sections: { name: string; entries: [string, string][] }[] = [];
  for (const line of text.split("\n")) {
    if (line.startsWith("[") && !line.includes("=")) sections.push({ name: line.slice(1, -1), entries: [] });
    else if (line.includes("=")) {
      const i = line.indexOf("=");
      sections[sections.length - 1].entries.push([line.slice(0, i), line.slice(i + 1)]);
    }
  }
  return { sections };
}

const profiles = [
  {
    id: "pr1",
    name: "Main — keybinds",
    game: "lol",
    createdAt: now - 10 * day,
    updatedAt: now - day,
    source: "client",
    categories: ["keybinds", "interface"],
    notes: "",
    lol: { input: parseIni(input), game: parseIni("[HUD]\nGlobalScale=0.5000\nShowTimestamps=1\nMinimapScale=1.2000\n[Volume]\nMasterVolume=0.6000") },
    val: null,
  },
];

let settings = {
  setupDone: true,
  language: "fr",
  theme: "liquid-glass-dark",
  performance: "quality",
  zoom: 100,
  streamerMode: false,
  checkUpdates: true,
  bonusShards: 0,
  playMenuLabels: true,
  autoLockMinutes: 0,
  lockOnSleep: false,
  closeToTray: true,
  startMinimized: false,
  blockSwitchInGame: true,
  closeRiotClient: true,
  rememberGhost: false,
  schedules: [],
  autoApplyProfiles: true,
  discordRpc: false,
  discordShowRiotId: false,
  discordOffWhenOffline: true,
  valorantStoreOptIn: false,
  thirdPartyRanksOptIn: false,
  telemetry: false,
  telemetryInstallId: "",
  lastGame: "lol",
  riotClientPath: "",
  lolPath: "",
};

const params = new URLSearchParams(location.search);
if (params.get("lang")) settings.language = params.get("lang")!;
if (params.get("theme")) settings.theme = params.get("theme")!;

export function installMock() {
  mockWindows("main");
  mockIPC((cmd, args) => {
    const a = (args ?? {}) as Record<string, unknown>;
    switch (cmd) {
      case "get_bootstrap":
        return {
          vaultExists: params.get("view") !== "setup",
          vaultMode: "password",
          unlocked: params.get("view") !== "unlock",
          settings,
          portable: false,
          version: "0.1.0",
          os: "windows",
          dataDir: "C:\\Users\\demo\\AppData\\Roaming\\RiftSwitch",
        };
      case "save_settings":
        settings = a.settings as typeof settings;
        return settings;
      case "riot_status":
        return {
          clientInstalled: true,
          procs: { riotClient: true, riotClientPid: 1, lolClient: true, lolGame: false, valGame: false, valInMatch: false },
          identity: { gameName: "Prince", tagLine: "EUW", puuid: "p1" },
          activeAccountId: "a1",
          proxyRunning: false,
          offline: false,
          chatStatus: null,
          busy: null,
        };
      case "list_accounts":
        return accounts;
      case "list_folders":
        return folders;
      case "list_profiles":
        return profiles;
      case "get_profile":
        return structuredClone(profiles.find((p) => p.id === a.id));
      case "list_val_dirs":
        return [{ name: "p3-eu", puuid: "p3", shard: "eu", modified: now, hasKeybinds: true }];
      case "runes_library":
        return [{ id: "r1", name: "Ahri électrocution", primaryStyleId: 8100, subStyleId: 8200, selectedPerkIds: [8112, 8139, 8138, 8135, 8226, 8210, 5008, 5008, 5001], champion: null, sourceAccount: "a1", createdAt: now }];
      case "runes_client_pages":
        throw { code: "lol_client_closed", detail: "" };
      case "play":
        return new Promise((r) => setTimeout(() => r({ switched: a.accountId !== "a1" }), 1200));
      case "telemetry_preview":
        return { installId: "(généré à l'activation)", accountsBucket: "2-5", hoursPlayedRounded: 70 };
      case "plugin:event|listen":
        return 0;
      default:
        return null;
    }
  });
}
