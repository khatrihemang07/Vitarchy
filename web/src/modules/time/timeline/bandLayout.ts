import type { SegmentView } from "@/api/time";

/** The narrowest a drawn mark gets (in % of the Day), so a short Segment can still be seen and pointed at. */
export const MIN_WIDTH_PCT = 0.3;

/** Segments narrower than this (in % of the Day) are merged with their neighbours for drawing. */
export const SLIVER_PCT = 0.15;

type Placed = Pick<SegmentView, "start_at" | "end_at" | "away">;

/** The Day a band covers, as instants. */
export interface DayBounds {
  startMs: number;
  endMs: number;
}

/** One drawn mark: where it sits and which Segments (indexes into the input) it stands for. */
export interface BandMark {
  left: number;
  width: number;
  segments: number[];
  /** The longest of its Segments, which sets the mark's colour and its tooltip. */
  main: number;
}

/** A local calendar Day from midnight to the next midnight (23 or 25 hours when the clocks change). */
export function dayBounds(day: string): DayBounds {
  const [y = 0, m = 1, d = 1] = day.split("-").map(Number);
  return { startMs: new Date(y, m - 1, d).getTime(), endMs: new Date(y, m - 1, d + 1).getTime() };
}

/** Where `ms` falls on the Day, in % (outside 0 to 100 when it is not on the Day). */
export const pctOf = (bounds: DayBounds, ms: number) => ((ms - bounds.startMs) / (bounds.endMs - bounds.startMs)) * 100;

interface Piece {
  index: number;
  away: Placed["away"];
  left: number;
  right: number;
}

function piecesOf(segments: readonly Placed[], bounds: DayBounds): Piece[] {
  const pieces: Piece[] = [];
  segments.forEach((segment, index) => {
    const left = Math.max(0, pctOf(bounds, Date.parse(segment.start_at)));
    const right = Math.min(100, pctOf(bounds, Date.parse(segment.end_at)));
    if (right > left) pieces.push({ index, away: segment.away, left, right });
  });
  return pieces;
}

/** Rounding slack (in % of the Day) when deciding that two Segments touch. */
const TOUCH_EPSILON = 1e-9;

const isSliver = (piece: Piece) => piece.right - piece.left < SLIVER_PCT;

/** A sliver joins a run of slivers only when it touches the run's end and shares its Away, so colour never covers time without data. */
function joinsRun(run: Piece[], piece: Piece): boolean {
  const end = run[run.length - 1] as Piece;
  return isSliver(piece) && run.every(isSliver) && piece.left - end.right < TOUCH_EPSILON && piece.away === end.away;
}

function markOf(group: Piece[]): BandMark {
  const first = group[0] as Piece;
  const last = group[group.length - 1] as Piece;
  const main = group.reduce((a, b) => (b.right - b.left > a.right - a.left ? b : a));
  return {
    left: first.left,
    width: Math.max(MIN_WIDTH_PCT, last.right - first.left),
    segments: group.map((p) => p.index),
    main: main.index,
  };
}

/**
 * Marks to draw for a Day's Segments, in order. Runs of slivers become one mark so
 * hundreds of tiny Segments do not paint as noise; every Segment stays in some mark.
 */
export function layoutBand(segments: readonly Placed[], bounds: DayBounds): BandMark[] {
  const groups: Piece[][] = [];
  for (const piece of piecesOf(segments, bounds)) {
    const last = groups[groups.length - 1];
    if (last && joinsRun(last, piece)) last.push(piece);
    else groups.push([piece]);
  }
  return groups.map(markOf);
}

/** An hour mark: where it sits (0 to 100) and the instant it marks. */
export interface HourTick {
  pct: number;
  date: Date;
}

/** Marks every `step` hours of local time from midnight to the next midnight; times the clocks skip are not repeated. */
export function hourTicks(day: string, step: number): HourTick[] {
  const bounds = dayBounds(day);
  const start = new Date(bounds.startMs);
  const ticks: HourTick[] = [];
  for (let hour = 0; hour <= 24; hour += step) {
    const date = new Date(start.getFullYear(), start.getMonth(), start.getDate(), hour);
    if (ticks.some((tick) => tick.date.getTime() === date.getTime())) continue;
    ticks.push({ pct: pctOf(bounds, date.getTime()), date });
  }
  return ticks;
}

/** Where `nowMs` falls inside the Day (0 to 100), or null when it is not that Day. */
export function nowPct(bounds: DayBounds, nowMs: number): number | null {
  return nowMs >= bounds.startMs && nowMs <= bounds.endMs ? pctOf(bounds, nowMs) : null;
}
