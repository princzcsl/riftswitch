export type Game = "lol" | "tft" | "val";

export interface Settings {
  setupDone: boolean;
  language: string;
  theme: string;
  performance: "auto" | "quality" | "performance";
  zoom: number;
  streamerMode: boolean;
  checkUpdates: boolean;
  bonusShards: number;
  playMenuLabels: boolean;
  autoLockMinutes: number;
  lockOnSleep: boolean;
  closeToTray: boolean;
  startMinimized: boolean;
  blockSwitchInGame: boolean;
  closeRiotClient: boolean;
  rememberGhost: boolean;
  autoApplyProfiles: boolean;
  discordRpc: boolean;
  discordShowRiotId: boolean;
  discordOffWhenOffline: boolean;
  valorantStoreOptIn: boolean;
  thirdPartyRanksOptIn: boolean;
  telemetry: boolean;
  telemetryInstallId: string;
  lastGame: Game;
  riotClientPath: string;
  lolPath: string;
}

export interface Bootstrap {
  vaultExists: boolean;
  vaultMode: "password" | "os" | null;
  unlocked: boolean;
  settings: Settings;
  portable: boolean;
  version: string;
  os: string;
  dataDir: string;
}

export interface Rank {
  tier: string;
  division: string;
  lp: number;
  wins: number;
  losses: number;
}

export interface Ranks {
  lolSolo: Rank | null;
  lolFlex: Rank | null;
  tft: Rank | null;
  val: Rank | null;
  updatedAt: number | null;
  manual: boolean;
}

export interface Account {
  id: string;
  label: string;
  gameName: string;
  tagLine: string;
  riotId: string;
  puuid: string | null;
  maskedEmail: string;
  region: string;
  color: string;
  icon: string;
  tags: string[];
  folderId: string | null;
  favorite: boolean;
  createdAt: number;
  lastUsedAt: number | null;
  lastGame: Game | null;
  langs: { lol: string | null; tft: string | null; val: string | null };
  twoFa: boolean;
  notes: string;
  championPool: string[];
  agentPool: string[];
  roles: string[];
  aliases: { riotId: string; seenAt: number }[];
  ranks: Ranks;
  decay: { daysRemaining: number | null; bankedGames: number | null; updatedAt: number | null };
  health: { penalties: string[]; honorLevel: number | null; updatedAt: number | null };
  playtime: { lol: number; tft: number; val: number; recent: Record<string, number>; launches: Record<string, number>; launcher?: { lol: number; tft: number; val: number } };
  profiles: { lol: string | null; val: string | null };
  sessionCapturedAt: number | null;
  hasSession: boolean;
  hasPassword: boolean;
  hasUsername: boolean;
  profileIconId: number | null;
  level: number | null;
  valShard: string | null;
}

export interface Folder {
  id: string;
  name: string;
  color: string;
  order: number;
}

export interface ProcSnapshot {
  riotClient: boolean;
  riotClientPid: number | null;
  lolClient: boolean;
  lolGame: boolean;
  valGame: boolean;
  valInMatch: boolean;
}

export interface RiotStatus {
  clientInstalled: boolean;
  procs: ProcSnapshot;
  identity: { gameName: string; tagLine: string; puuid: string | null } | null;
  activeAccountId: string | null;
  proxyRunning: boolean;
  offline: boolean;
  chatStatus: string | null;
  busy: "switching" | "waiting_login" | null;
}

export type IniEntry = [string, string];
export interface IniSection {
  name: string;
  entries: IniEntry[];
}
export interface IniDoc {
  sections: IniSection[];
}

export interface ConfigProfile {
  id: string;
  name: string;
  game: "lol" | "val";
  createdAt: number;
  updatedAt: number;
  source: string;
  categories: string[];
  notes: string;
  lol: { input: IniDoc; game: IniDoc } | null;
  val: { files: Record<string, string> } | null;
}

export interface ValDir {
  name: string;
  puuid: string;
  shard: string;
  modified: number;
  hasKeybinds: boolean;
}

export interface RunePage {
  id: string;
  name: string;
  primaryStyleId: number;
  subStyleId: number;
  selectedPerkIds: number[];
  champion: string | null;
  sourceAccount: string | null;
  createdAt: number;
}

export interface AppError {
  code: string;
  detail: string;
}

export interface AccountInput {
  id?: string | null;
  label: string;
  gameName: string;
  tagLine: string;
  region: string;
  color: string;
  icon: string;
  tags: string[];
  folderId: string | null;
  favorite: boolean;
  langs: { lol: string | null; tft: string | null; val: string | null };
  twoFa: boolean;
  notes: string;
  championPool: string[];
  agentPool: string[];
  roles: string[];
  profiles: { lol: string | null; val: string | null };
  ranks?: Ranks | null;
  email?: string | null;
  username?: string | null;
  password?: string | null;
}
