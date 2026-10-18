// « Sur quel compte jouer ce soir ? » : score explicable selon l'objectif choisi.
import type { Account, Game, Rank } from "./types";
import { rankScore, recentPlaytime, TIERS } from "./format";

export type Goal = "grind" | "role" | "friends" | "decay";

export interface Suggestion {
  account: Account;
  score: number;
  reasons: { key: string; params?: Record<string, string | number>; good: boolean }[];
}

function rankFor(a: Account, game: Game): Rank | null {
  return game === "val" ? a.ranks.val : game === "tft" ? a.ranks.tft : a.ranks.lolSolo;
}

export function suggest(
  accounts: Account[],
  goal: Goal,
  opts: { game: Game; role?: string; friendTier?: string },
): Suggestion[] {
  const out: Suggestion[] = accounts.map((account) => {
    let score = 50;
    const reasons: Suggestion["reasons"] = [];
    const r = rankFor(account, opts.game);
    const recent = recentPlaytime(account, 7);

    if (!account.hasSession) {
      score -= 60;
      reasons.push({ key: "tonight.r_no_session", good: false });
    }
    if (account.health.penalties.length) {
      score -= 40;
      reasons.push({ key: "tonight.r_penalty", good: false });
    }

    switch (goal) {
      case "grind": {
        // Le meilleur compte classé, légèrement favorisé s'il a été peu joué récemment (fraîcheur).
        const s = rankScore(r, opts.game === "val");
        score += s >= 0 ? s / 40 : -10;
        if (s >= 0) reasons.push({ key: "tonight.r_rank", params: { rank: r?.tier ?? "" }, good: true });
        if (recent < 3 * 3600) {
          score += 8;
          reasons.push({ key: "tonight.r_fresh", good: true });
        } else if (recent > 15 * 3600) {
          score -= 8;
          reasons.push({ key: "tonight.r_tired", params: { h: Math.round(recent / 3600) }, good: false });
        }
        break;
      }
      case "role": {
        const role = (opts.role ?? "").toLowerCase();
        const mains = account.roles.map((x) => x.toLowerCase());
        if (role && mains.includes(role)) {
          score -= 25;
          reasons.push({ key: "tonight.r_main_role", good: false });
        } else {
          score += 15;
          reasons.push({ key: "tonight.r_off_role", good: true });
        }
        // Un compte moins haut classé limite l'impact d'un apprentissage.
        const s = rankScore(r, opts.game === "val");
        score -= s >= 0 ? s / 80 : 0;
        if (account.tags.some((t) => /smurf|learn|test|appr/i.test(t))) {
          score += 12;
          reasons.push({ key: "tonight.r_smurf_tag", good: true });
        }
        break;
      }
      case "friends": {
        const target = TIERS.indexOf((opts.friendTier ?? "GOLD").toUpperCase());
        const mine = r?.tier ? TIERS.indexOf(r.tier) : -1;
        if (mine < 0) {
          score -= 5;
          reasons.push({ key: "tonight.r_unranked", good: false });
        } else {
          const gap = Math.abs(mine - target);
          score += 30 - gap * 15;
          reasons.push({ key: gap <= 1 ? "tonight.r_duo_ok" : "tonight.r_duo_gap", params: { n: gap }, good: gap <= 1 });
        }
        break;
      }
      case "decay": {
        const d = account.decay.daysRemaining;
        if (d != null) {
          score += 60 - d * 4;
          reasons.push({ key: "tonight.r_decay", params: { n: d }, good: true });
        } else {
          score -= 20;
          reasons.push({ key: "tonight.r_no_decay", good: false });
        }
        break;
      }
    }
    return { account, score: Math.round(score), reasons };
  });
  return out.sort((a, b) => b.score - a.score);
}
