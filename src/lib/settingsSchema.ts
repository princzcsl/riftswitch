// Réglages connus des profils de config, rangés comme dans les menus des jeux. Libellés : `cfgk.<clé>`,
// choix : `cfgopt.<liste>_<valeur>`, groupes : `configs.g_<groupe>`. Le reste s'affiche dans « Avancé ».

export type Field = {
  key: string;
  kind: "bool" | "slider" | "select" | "number" | "key" | "mapSkins";
  min?: number;
  max?: number;
  step?: number;
  /** Valeur 0–1 affichée en pourcentage. */
  pct?: boolean;
  /** Clé « muet » associée à un volume (LoL). */
  mute?: string;
  options?: number[];
  list?: string;
  /** Libellé `cfgk.<label>` quand la clé seule est ambiguë entre les deux jeux. */
  label?: string;
};
export type Group = { id: string; fields: Field[] };

const bools = (...keys: string[]): Field[] => keys.map((key) => ({ key, kind: "bool" }));
const vol = (key: string, mute?: string): Field => ({ key, kind: "slider", min: 0, max: 1, step: 0.01, pct: true, mute });
const quality = (list: string, n: number) => (key: string): Field => ({ key, kind: "select", options: [...Array(n).keys()], list });

export const LOL_SCHEMA: Record<string, Group[]> = {
  interface: [
    {
      id: "hud",
      fields: [
        { key: "GlobalScale", kind: "slider", min: 0, max: 1, step: 0.01, pct: true },
        { key: "MinimapScale", kind: "slider", min: 0, max: 3, step: 0.01 },
        { key: "ChatScale", kind: "slider", min: 0, max: 100, step: 1 },
        ...bools(
          "ShowFPSAndLatency",
          "ShowTimestamps",
          "DrawHealthBars",
          "ShowSummonerNames",
          "ShowAttackRadius",
          "ShowNeutralCamps",
          "ShowSpellCosts",
          "ShowSpellRecommendations",
          "ShowTeamFramesOnLeft",
          "MirroredScoreboard",
          "FlipMiniMap",
          "MinimapMoveSelf",
          "MinimapEnableAllTimers",
          "ShowOffScreenPointsOfInterest",
          "FlashScreenWhenDamaged",
          "FlashScreenWhenStunned",
          "EnableLineMissileVis",
          "AutoDisplayTarget",
          "ShowAlliedChat",
          "ShowAllChannelChat",
          "HideEnemySummonerEmotes",
          "ShowPlayerStats",
        ),
      ],
    },
    {
      id: "mouse",
      fields: [
        { key: "GameMouseSpeed", kind: "slider", min: 1, max: 20, step: 1 },
        { key: "CursorScale", kind: "slider", min: 0, max: 1, step: 0.01, pct: true },
        ...bools("ShowCursorLocator", "EnableTargetedAttackMove", "TargetChampionsOnlyAsToggle", "AutoAcquireTarget", "PredictMovement", "ClampCastTargetLocationWithinMaxRange", "SmartCastOnKeyRelease"),
      ],
    },
    {
      id: "camera",
      fields: [
        { key: "MapScrollSpeed", kind: "slider", min: 0, max: 1, step: 0.01, pct: true },
        { key: "KeyboardScrollSpeed", kind: "slider", min: 0, max: 1, step: 0.01, pct: true },
        ...bools("ScrollSmoothingEnabled", "MiddleClickDragScrollEnabled", "SnapCameraOnRespawn", "MinimizeCameraMotion"),
      ],
    },
    {
      id: "gameplay",
      fields: bools("RelativeTeamColors", "EnableChampionSpellPreview", "AlwaysShowExtendedTooltip", "ShowTurretRangeIndicators", "RecommendJunglePaths", "EnableGlobalSpellCastNotifications", "LossOfControlEnabled", "ShowSlows"),
    },
    { id: "map", fields: [{ key: "MapSkinOptionDisable", kind: "mapSkins" }] },
    {
      id: "floatingText",
      fields: bools("Damage_Enabled", "EnemyDamage_Enabled", "Heal_Enabled", "ManaDamage_Enabled", "Gold_Enabled", "Experience_Enabled", "Level_Enabled", "Dodge_Enabled", "Invulnerable_Enabled", "QuestReceived_Enabled", "Score_Enabled", "Special_Enabled"),
    },
  ],
  sound: [
    {
      id: "volume",
      fields: [
        vol("MasterVolume", "MasterMute"),
        vol("MusicVolume", "MusicMute"),
        vol("AnnouncerVolume", "AnnouncerMute"),
        vol("VoiceVolume", "VoiceMute"),
        vol("SfxVolume", "SfxMute"),
        vol("AmbienceVolume", "AmbienceMute"),
        vol("PingsVolume", "PingsMute"),
        vol("NotificationsVolume", "NotificationsMute"),
        ...bools("EnableAudio", "EnableCustomAnnouncer"),
      ],
    },
    { id: "voice", fields: [vol("InputVolume"), vol("ActivationSensitivity"), ...bools("ShowVoiceChatHalos", "ShowVoicePanelWithScoreboard")] },
  ],
  graphics: [
    {
      id: "display",
      fields: [
        { key: "WindowMode", kind: "select", options: [0, 1, 2], list: "lolWindow" },
        { key: "Width", kind: "number" },
        { key: "Height", kind: "number" },
        ...bools("WaitForVerticalSync"),
      ],
    },
    {
      id: "quality",
      fields: [...["CharacterQuality", "EnvironmentQuality", "EffectsQuality", "ShadowQuality"].map(quality("lolQuality", 5)), ...bools("EnableFXAA", "EnableHUDAnimations", "ShowGodray", "HideEyeCandy")],
    },
  ],
};

export const VAL_SCHEMA: Record<string, Group[]> = {
  interface: [
    {
      id: "mouse",
      fields: [
        { key: "MouseSensitivity", kind: "number", step: 0.001 },
        { key: "MouseSensitivityADS", kind: "number", step: 0.001 },
        { key: "MouseSensitivityZoomed", kind: "number", step: 0.001 },
        ...bools("InvertMouse"),
      ],
    },
    {
      id: "minimap",
      fields: [{ key: "MinimapSize", kind: "number", step: 0.001 }, { key: "MinimapZoom", kind: "number", step: 0.001 }, ...bools("MinimapRotates", "MinimapTranslates")],
    },
    {
      id: "gameplay",
      fields: [
        { key: "ColorBlindMode", kind: "select", options: [0, 1, 2, 3], list: "colorBlind" },
        ...bools("ShowBlood", "ShowCorpses", "ShowBulletTracers", "AutoEquipPrioritizeStrongest", "MuteMusicOnAppWindowDeactivate", "PlayerBehavior_MuteEnemyTextChat"),
      ],
    },
  ],
  sound: [
    {
      id: "volume",
      fields: ["OverallVolume", "SoundEffectsVolume", "VoiceOverVolume", "VideoVolume", "AllMusicOverallVolume", "MenuAndLobbyMusicVolume", "CharacterSelectMusicVolume"].map((k) => vol(k)).concat(bools("EnableHRTF")),
    },
    {
      id: "voice",
      fields: [
        ...bools("VoiceChatEnabled", "PushToTalkEnabled", "VoipDucksMusicVolume"),
        { key: "PushToTalkKey", kind: "key" },
        { key: "TeamPushToTalkKey", kind: "key" },
        { key: "VoiceVolume", kind: "slider", min: 0, max: 100, step: 1, label: "VoiceChatVolume" },
        { key: "MicVolume", kind: "slider", min: 0, max: 100, step: 1 },
        { key: "MicSensitivityThreshold", kind: "slider", min: 0, max: 100, step: 1 },
      ],
    },
  ],
  graphics: [
    {
      id: "display",
      fields: [
        { key: "PreferredFullscreenMode", kind: "select", options: [0, 1, 2], list: "ueWindow" },
        { key: "ResolutionSizeX", kind: "number" },
        { key: "ResolutionSizeY", kind: "number" },
        { key: "FrameRateLimit", kind: "number" },
        ...bools("bUseVSync", "bShouldLetterbox"),
      ],
    },
    {
      id: "quality",
      fields: ["sg.TextureQuality", "sg.EffectsQuality", "sg.ShadowQuality", "sg.AntiAliasingQuality", "sg.PostProcessQuality", "sg.ViewDistanceQuality", "sg.FoliageQuality", "sg.ShadingQuality", "sg.ReflectionQuality", "sg.GlobalIlluminationQuality"].map(quality("valQuality", 4)),
    },
  ],
};

/** Clé sans préfixe VALORANT (« EAresFloatSettingName::X » → « X »). */
export const bareKey = (k: string) => k.split("::").pop() ?? k;

export function schemaKeys(groups: Group[]): Set<string> {
  const out = new Set<string>();
  for (const g of groups) for (const f of g.fields) (out.add(f.key), f.mute && out.add(f.mute));
  return out;
}
