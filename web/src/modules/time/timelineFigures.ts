import type { TimelineStatus } from "@/api/time";

/** What the Timeline card shows, derived from the stored figures. */
export interface TimelineFigures {
  /** Away and unknown-Away as whole-percent shares of the Timeline's covered time, or null when there is none. */
  awayShare: number | null;
  unknownShare: number | null;
}

/** Whole percent of `part` in `total`, or null when `total` is not positive. */
export function sharePercent(part: number, total: number): number | null {
  if (total <= 0) return null;
  return Math.round((Math.min(part, total) / total) * 100);
}

/**
 * Derives the shares. They are of covered time (the sum of Segments), which Away and
 * unknown-Away seconds lie inside, so gaps between Segments count for nothing.
 */
export function timelineFigures(timeline: TimelineStatus): TimelineFigures {
  return {
    awayShare: sharePercent(timeline.away_secs, timeline.covered_secs),
    unknownShare: sharePercent(timeline.unknown_secs, timeline.covered_secs),
  };
}
