// Catalogue des actions de jeu affichées dans l'éditeur de touches (libellés en/fr, repli en anglais).
// Les actions absentes du catalogue restent éditables dans « Toutes les actions ».

export interface ActionDef {
  id: string;
  group: string;
  en: string;
  fr: string;
}

const L = (id: string, group: string, en: string, fr: string): ActionDef => ({ id, group, en, fr });

export const LOL_GROUPS = ["spells", "items", "movement", "camera", "communication", "interface", "emotes"] as const;

export const LOL_ACTIONS: ActionDef[] = [
  L("evtCastSpell1", "spells", "Ability Q", "Compétence Q"),
  L("evtCastSpell2", "spells", "Ability W", "Compétence W"),
  L("evtCastSpell3", "spells", "Ability E", "Compétence E"),
  L("evtCastSpell4", "spells", "Ultimate R", "Ultime R"),
  L("evtCastAvatarSpell1", "spells", "Summoner spell 1 (D)", "Sort d'invocateur 1 (D)"),
  L("evtCastAvatarSpell2", "spells", "Summoner spell 2 (F)", "Sort d'invocateur 2 (F)"),
  L("evtCastRoleBound", "spells", "Role quest item", "Objet de quête de rôle"),
  L("evtLevelSpell1", "spells", "Level up Q", "Améliorer Q"),
  L("evtLevelSpell2", "spells", "Level up W", "Améliorer W"),
  L("evtLevelSpell3", "spells", "Level up E", "Améliorer E"),
  L("evtLevelSpell4", "spells", "Level up R", "Améliorer R"),
  L("evtSmartCastSpell1", "spells", "Quick cast Q", "Incantation rapide Q"),
  L("evtSmartCastSpell2", "spells", "Quick cast W", "Incantation rapide W"),
  L("evtSmartCastSpell3", "spells", "Quick cast E", "Incantation rapide E"),
  L("evtSmartCastSpell4", "spells", "Quick cast R", "Incantation rapide R"),
  L("evtSelfCastSpell1", "spells", "Self cast Q", "Auto-incantation Q"),
  L("evtSelfCastSpell2", "spells", "Self cast W", "Auto-incantation W"),
  L("evtSelfCastSpell3", "spells", "Self cast E", "Auto-incantation E"),
  L("evtSelfCastSpell4", "spells", "Self cast R", "Auto-incantation R"),
  L("evtUseItem1", "items", "Item slot 1", "Objet 1"),
  L("evtUseItem2", "items", "Item slot 2", "Objet 2"),
  L("evtUseItem3", "items", "Item slot 3", "Objet 3"),
  L("evtUseItem4", "items", "Item slot 4", "Objet 4"),
  L("evtUseItem5", "items", "Item slot 5", "Objet 5"),
  L("evtUseItem6", "items", "Item slot 6", "Objet 6"),
  L("evtUseVisionItem", "items", "Trinket", "Balise"),
  L("evtUseItem7", "items", "Recall", "Rappel"),
  L("evtOpenShop", "items", "Open shop", "Ouvrir la boutique"),
  L("evtPlayerMoveClick", "movement", "Move", "Déplacement"),
  L("evtPlayerAttackMove", "movement", "Attack move", "Attaque-déplacement"),
  L("evtPlayerAttackMoveClick", "movement", "Attack move click", "Clic d'attaque-déplacement"),
  L("evtPlayerAttackOnlyClick", "movement", "Attack only", "Attaquer uniquement"),
  L("evtPlayerHoldPosition", "movement", "Hold position", "Tenir la position"),
  L("evtPlayerStopPosition", "movement", "Stop", "Arrêt"),
  L("evtChampionOnly", "movement", "Target champions only", "Cibler les champions uniquement"),
  L("evtPetMoveClick", "movement", "Pet move", "Déplacer le familier"),
  L("evtCameraLockToggle", "camera", "Camera lock toggle", "Verrouiller la caméra"),
  L("evtCameraSnap", "camera", "Center camera", "Centrer la caméra"),
  L("evtSelectSelf", "camera", "Select self", "Focus sur soi"),
  L("evtSelectAlly1", "camera", "Select ally 1", "Sélectionner l'allié 1"),
  L("evtSelectAlly2", "camera", "Select ally 2", "Sélectionner l'allié 2"),
  L("evtSelectAlly3", "camera", "Select ally 3", "Sélectionner l'allié 3"),
  L("evtSelectAlly4", "camera", "Select ally 4", "Sélectionner l'allié 4"),
  L("evtScrollUp", "camera", "Scroll camera up", "Caméra vers le haut"),
  L("evtScrollDown", "camera", "Scroll camera down", "Caméra vers le bas"),
  L("evtScrollLeft", "camera", "Scroll camera left", "Caméra vers la gauche"),
  L("evtScrollRight", "camera", "Scroll camera right", "Caméra vers la droite"),
  L("evntPlayerPing", "communication", "Ping", "Signal"),
  L("evntPlayerPingDanger", "communication", "Danger ping", "Signal de danger"),
  L("evntPlayerPingCursor", "communication", "Ping at cursor", "Signal au curseur"),
  L("evntPlayerPingCursorDanger", "communication", "Danger ping at cursor", "Signal de danger au curseur"),
  L("evtPlayerPingRadialDanger", "communication", "Radial ping menu", "Roue des signaux"),
  L("evtPlayerPingOMW", "communication", "On my way", "J'arrive"),
  L("evtPlayerPingMIA", "communication", "Enemy missing", "Ennemi disparu"),
  L("evtPlayerPingAllIn", "communication", "All in", "Engagez le combat"),
  L("evtPlayerPingPush", "communication", "Push", "Push"),
  L("evtPlayerPingComeHere", "communication", "Assist me", "Aidez-moi"),
  L("evtPlayerPingVisionNeeded", "communication", "Need vision", "Besoin de vision"),
  L("evtChatHistory", "communication", "Chat history", "Historique du chat"),
  L("evtPushToTalk", "communication", "Push to talk", "Appuyer pour parler"),
  L("evtPushToTalkTeam", "communication", "Push to talk (team)", "Appuyer pour parler (équipe)"),
  L("evtShowVoicePanel", "communication", "Voice panel", "Panneau vocal"),
  L("evtShowScoreBoard", "interface", "Scoreboard", "Tableau des scores"),
  L("evtHoldShowScoreBoard", "interface", "Hold scoreboard", "Maintenir le tableau des scores"),
  L("evtTogglePlayerStats", "interface", "Toggle champion stats", "Alterner statistiques champion"),
  L("evtToggleFPSAndLatency", "interface", "FPS & ping display", "Affichage FPS et latence"),
  L("evtToggleMinionHealthBars", "interface", "Minion health bars", "Barres de vie des sbires"),
  L("evtShowHealthBars", "interface", "Health bars", "Barres de vie"),
  L("evtShowSummonerNames", "interface", "Summoner names", "Noms d'invocateur"),
  L("evtDrawHud", "interface", "Toggle HUD", "Afficher / masquer l'interface"),
  L("evtSysMenu", "interface", "Menu", "Menu"),
  L("evtShowCharacterMenu", "interface", "Champion stats", "Statistiques du champion"),
  L("evtToggleMouseClip", "interface", "Lock mouse in window", "Bloquer la souris dans la fenêtre"),
  L("evtEmoteDance", "emotes", "Dance", "Danse"),
  L("evtEmoteJoke", "emotes", "Joke", "Blague"),
  L("evtEmoteTaunt", "emotes", "Taunt", "Provocation"),
  L("evtEmoteLaugh", "emotes", "Laugh", "Rire"),
  L("evtEmoteToggle", "emotes", "Toggle emotes", "Basculer les emotes"),
  L("evtChampMasteryDisplay", "emotes", "Mastery emote", "Emote de maîtrise"),
  L("evtRadialEmoteOpen", "emotes", "Emote wheel", "Roue des emotes"),
];

export const VAL_GROUPS = ["movement", "equipment", "abilities", "communication", "interface"] as const;

export const VAL_ACTIONS: ActionDef[] = [
  // Déplacements : axes VALORANT (touche + sens), identifiants internes à RiftSwitch.
  L("axis:MoveForward:1", "movement", "Move forward", "Avancer"),
  L("axis:MoveForward:-1", "movement", "Move backward", "Reculer"),
  L("axis:MoveRight:-1", "movement", "Strafe left", "Gauche"),
  L("axis:MoveRight:1", "movement", "Strafe right", "Droite"),
  L("Jump", "movement", "Jump", "Sauter"),
  L("Crouch", "movement", "Crouch", "S'accroupir"),
  L("Walk", "movement", "Walk", "Marcher"),
  L("Activate_Primary", "equipment", "Primary weapon", "Arme principale"),
  L("Activate_Secondary", "equipment", "Secondary weapon", "Arme secondaire"),
  L("Activate_Melee", "equipment", "Melee", "Arme de mêlée"),
  L("Activate_Level", "equipment", "Spike", "Spike"),
  L("NextWeapon", "equipment", "Next weapon", "Arme suivante"),
  L("PrevWeapon", "equipment", "Previous weapon", "Arme précédente"),
  L("Reload", "equipment", "Reload", "Recharger"),
  L("UseObject", "equipment", "Use / plant", "Utiliser / poser"),
  L("UseAlternateObject", "equipment", "Alternate use", "Utilisation alternative"),
  L("Inspect", "equipment", "Inspect weapon", "Inspecter l'arme"),
  L("DropEquippable", "equipment", "Drop", "Lâcher"),
  L("Activate_Ability1", "abilities", "Ability 1", "Compétence 1"),
  L("Activate_Ability2", "abilities", "Ability 2", "Compétence 2"),
  L("Activate_GrenadeAbility", "abilities", "Ability 3", "Compétence 3"),
  L("Activate_Ultimate", "abilities", "Ultimate", "Ultime"),
  L("Ping", "communication", "Ping", "Signal"),
  L("OpenCommMenu", "communication", "Comms wheel", "Roue de communication"),
  L("VOICE_TeamPTTAction", "communication", "Voice Team", "Voice Team"),
  L("VOICE_PartyPTTAction", "communication", "Voice Party", "Voice Party"),
  L("VOICE_TeamClutchMuteAction", "communication", "Clutch mute", "Mettre en sourdine (clutch)"),
  L("VoteOption0", "communication", "Vote option 1", "Option de vote 1"),
  L("VoteOption1", "communication", "Vote option 2", "Option de vote 2"),
  L("VoteOption2", "communication", "Vote option 3", "Option de vote 3"),
  L("VoteOption3", "communication", "Vote option 4", "Option de vote 4"),
  L("OpenMegamap", "interface", "Map", "Carte"),
  L("ShowScoreboard", "interface", "Scoreboard", "Tableau des scores"),
  L("OpenBuyMenu", "interface", "Buy menu", "Menu d'achat"),
  L("Ghost", "interface", "Spectate / ghost", "Spectateur"),
  L("ToggleMouseCursor", "interface", "Toggle cursor", "Afficher le curseur"),
];

// Variantes d'incantation de LoL (evtSmartCastItem3…) : mode + action de base.
const CAST_MODES: [string, string, string][] = [
  ["SmartPlusSelfCastWithIndicator", "Incantation rapide + sur soi avec indicateur", "Quick cast + self cast with indicator"],
  ["SmartPlusSelfCast", "Incantation rapide + sur soi", "Quick cast + self cast"],
  ["SmartCastWithIndicator", "Incantation rapide avec indicateur", "Quick cast with indicator"],
  ["SmartCast", "Incantation rapide", "Quick cast"],
  ["SelfCast", "Sur soi", "Self cast"],
  ["NormalCast", "Lancement normal", "Normal cast"],
];
const CAST_TARGET = (t: string) => (/^(Spell\d|AvatarSpell\d|RoleBound)$/.test(t) ? `evtCast${t}` : `evtUse${t}`);
const OTHER: Record<string, [string, string]> = {
  evtRadialEmoteInstantOpen: ["Ouvrir la roue d'emotes", "Open emote wheel"],
  evtReciprocityTrigger: ["Remerciement", "Thanks"],
  evtReciprocityMyBadTrigger: ["Excuse", "My bad"],
  evtOnUIMouse4Pan: ["Déplacer la caméra (souris)", "Pan camera (mouse)"],
  evtDragScrollLock: ["Verrouiller le défilement", "Drag scroll lock"],
  evtToggleDeathRecapShowcase: ["Récapitulatif de mort", "Death recap"],
  evtShopSwitchTabs: ["Boutique : changer d'onglet", "Shop: switch tabs"],
  evtShopFocusSearch: ["Boutique : rechercher", "Shop: search"],
};

function composedLabel(list: ActionDef[], id: string, fr: boolean): string | null {
  if (OTHER[id]) return OTHER[id][fr ? 0 : 1];
  const slot = id.match(/^evtRadialEmotePlaySlot(\d)$/);
  if (slot) return fr ? `Emote de la roue ${Number(slot[1]) + 1}` : `Wheel emote ${Number(slot[1]) + 1}`;
  const rest = id.replace(/^evt/, "");
  for (const [mode, f, e] of CAST_MODES) {
    if (!rest.startsWith(mode)) continue;
    const base = list.find((a) => a.id === CAST_TARGET(rest.slice(mode.length)));
    if (base) return `${fr ? f : e} : ${fr ? base.fr : base.en}`;
  }
  return null;
}

export function actionLabel(list: ActionDef[], id: string, lang: string): string {
  const def = list.find((a) => a.id === id);
  if (!def) return composedLabel(list, id, lang.startsWith("fr")) ?? humanize(id);
  return lang.startsWith("fr") ? def.fr : def.en;
}

export function humanize(id: string): string {
  return id
    .replace(/^evn?t/, "")
    .replace(/(PTT)?Action$/, "")
    .replace(/_/g, " ")
    .replace(/([a-z])([A-Z0-9])/g, "$1 $2")
    .trim();
}
