/** How hot a streak looks. */
export type StreakTier = "cold" | "warm" | "hot";

/** A week or more keeps the flame hot. */
export const HOT_STREAK_DAYS = 7;

export function streakTier(days: number): StreakTier {
  if (days >= HOT_STREAK_DAYS) return "hot";
  return days > 0 ? "warm" : "cold";
}

/** Human label for a streak; the cold tier has no day count to show. */
export function streakLabel(days: number): string {
  return streakTier(days) === "cold" ? "No streak yet" : `${String(days)}-day streak`;
}
