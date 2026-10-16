/** Placing a Day's segments on a lane and colouring them. Pure; shared by the Activity and Life log timelines. */
import { OTHER_COLOR, SERIES_COUNT, seriesColor } from "@/design";

/** A stretch of the Day with a name; `start` and `end` are ISO instants. */
export interface TimelineSegment {
  start: string;
  end: string;
  /** What it is, and the key its colour is chosen by. */
  label: string;
  /** Extra line for the tooltip, e.g. the app. */
  detail?: string;
}

/** A segment placed on the lane, in percent of the Day's length. */
export interface PlacedSegment {
  segment: TimelineSegment;
  left: number;
  width: number;
}

const ms = (iso: string) => new Date(iso).getTime();

/** Where each segment sits between `dayStart` and `dayEnd`; parts outside the Day are cut off. */
export function placeSegments(segments: readonly TimelineSegment[], dayStart: string, dayEnd: string): PlacedSegment[] {
  const from = ms(dayStart);
  const length = ms(dayEnd) - from;
  if (length <= 0) return [];
  return segments.flatMap((segment) => {
    const start = Math.max(ms(segment.start), from);
    const end = Math.min(ms(segment.end), ms(dayEnd));
    if (end <= start) return [];
    return [{ segment, left: ((start - from) / length) * 100, width: ((end - start) / length) * 100 }];
  });
}

/** Seconds in a segment. */
export function segmentSeconds(segment: TimelineSegment): number {
  return Math.round((ms(segment.end) - ms(segment.start)) / 1000);
}

/** The hours the lane labels, from midnight to midnight. */
export const TICK_HOURS = [0, 6, 12, 18, 24] as const;

/** A label's colour and the seconds it covers. */
export interface LegendEntry {
  label: string;
  color: string;
  seconds: number;
}

/**
 * Colours by label, longest total first: the longest labels take the palette in order and the
 * rest share the neutral "Other", so a Day never needs more than the palette holds.
 */
export function colorLabels(segments: readonly TimelineSegment[]): LegendEntry[] {
  const totals = new Map<string, number>();
  for (const segment of segments) totals.set(segment.label, (totals.get(segment.label) ?? 0) + segmentSeconds(segment));
  return [...totals]
    .sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0]))
    .map(([label, seconds], index) => ({ label, seconds, color: index < SERIES_COUNT ? seriesColor(index) : OTHER_COLOR }));
}
