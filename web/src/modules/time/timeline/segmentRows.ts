import type { SegmentView } from "@/api/time";
import { formatSpan } from "../format";
import { type Clock, formatClock } from "../reports/entryTimes";

/** The site of a page address without `www.`; null when there is none or it is not a web page. */
export function domainOf(url: string | null | undefined): string | null {
  if (!url) return null;
  try {
    const parsed = new URL(url);
    if (parsed.protocol !== "http:" && parsed.protocol !== "https:") return null;
    return parsed.hostname.replace(/^www\./, "");
  } catch {
    return null;
  }
}

/** How long a Segment lasts, in seconds. */
export function durationSecs(segment: Pick<SegmentView, "start_at" | "end_at">): number {
  return Math.round((Date.parse(segment.end_at) - Date.parse(segment.start_at)) / 1000);
}

/** True when the Segment belongs to the highlighted app (or nothing is highlighted). Away time has no app to match. */
export function matchesApp(segment: Pick<SegmentView, "app" | "away">, app: string | undefined): boolean {
  return app === undefined || (segment.away !== "yes" && segment.app === app);
}

/** What to call a Segment: its app, or a placeholder when the Source named none. Away is shown beside it, not instead. */
export function segmentLabel(segment: Pick<SegmentView, "app">): string {
  return segment.app || "(no app)";
}

/** "09:00 – 09:30 · 30m": when a Segment covers and for how long. */
export function segmentWhen(segment: Pick<SegmentView, "start_at" | "end_at">, clock: Clock): string {
  const start = formatClock(new Date(segment.start_at), clock);
  const end = formatClock(new Date(segment.end_at), clock);
  return `${start} – ${end} · ${formatSpan(durationSecs(segment))}`;
}
