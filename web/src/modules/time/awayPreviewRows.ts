import type { AwayPreview, PreviewChoice } from "@/api/time";
import { formatSpan } from "./format";

/** One line of the Away rules table, already written for the person. */
export interface AwayPreviewRow {
  key: number;
  /** The threshold: "5 min", "1 hour". */
  label: string;
  /** True for the threshold in force. */
  current: boolean;
  away: string;
  active: string;
  /** How Away differs from the current threshold: "+4h", "−2h", "same", or "current". */
  awayChange: string;
  activeChange: string;
}

const MINUTE = 60;
const HOUR = 60 * MINUTE;

function thresholdLabel(secs: number): string {
  return secs % HOUR === 0 && secs > 0 ? `${String(secs / HOUR)} ${secs === HOUR ? "hour" : "hours"}` : `${String(Math.round(secs / MINUTE))} min`;
}

function signedChange(delta: number): string {
  if (delta === 0) return "same";
  return `${delta > 0 ? "+" : "−"}${formatSpan(delta)}`;
}

function row(choice: PreviewChoice, current: PreviewChoice | undefined, isCurrent: boolean): AwayPreviewRow {
  const change = (value: number, against: number | undefined) =>
    isCurrent ? "current" : against === undefined ? "" : signedChange(value - against);
  return {
    key: choice.away_after_secs,
    label: thresholdLabel(choice.away_after_secs),
    current: isCurrent,
    away: formatSpan(choice.away_secs_per_day),
    active: formatSpan(choice.active_secs_per_day),
    awayChange: change(choice.away_secs_per_day, current?.away_secs_per_day),
    activeChange: change(choice.active_secs_per_day, current?.active_secs_per_day),
  };
}

/** The table rows: each threshold's Away and Active per Day, and their change from the current threshold. */
export function awayPreviewRows(preview: AwayPreview): AwayPreviewRow[] {
  const current = preview.choices.find((c) => c.away_after_secs === preview.current_secs);
  return preview.choices.map((choice) => row(choice, current, choice.away_after_secs === preview.current_secs));
}

/** The watching apps and sites, as one list for the card to name. */
export function watchingPlaces(preview: AwayPreview): string {
  return [...preview.watching_apps, ...preview.watching_sites].join(", ");
}

/** False when the preview window holds no Clockify Activity, so the table would be all zeros. */
export function hasPreviewData(preview: AwayPreview): boolean {
  return preview.choices.some((c) => c.away_secs_per_day + c.active_secs_per_day > 0);
}
