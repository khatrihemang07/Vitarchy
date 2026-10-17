import { formatCount, formatSpan, formatSpanRange } from "../format";
import { USUAL_SPAN } from "../usualWords";
import type { Figure, GroupFigures } from "./groupFigures";
import type { StatMode } from "./reportSearch";

/** What one table cell shows: text, and the change badge beneath it when there is one. */
export interface Cell {
  text: string;
  change?: number | null;
  /** Bold, for the total. */
  emphasis?: boolean;
}

/** What a cell may need to know about its table. */
export interface CellContext {
  /** Days of the Period so far, for "4 of 6". */
  periodDays: number;
}

/** One column: its header, the tooltip that explains it, and how it reads a row's figures. */
export interface Column {
  id: string;
  label: string;
  tooltip: string;
  cell: (figures: GroupFigures, context: CellContext) => Cell;
}

const NONE = "—";

const span = (figure: Figure, emphasis = false): Cell => ({ text: formatSpan(figure.value), change: figure.change, emphasis });

export const DAYS: Column = {
  id: "days",
  label: "Days",
  tooltip: "The Days this group had time, out of the Days of the Period so far",
  cell: (f, c) => ({ text: `${formatCount(f.days)} of ${formatCount(c.periodDays)}` }),
};

export const ENTRIES: Column = {
  id: "entries",
  label: "Entries",
  tooltip: "The Time Entries in this group",
  cell: (f) => ({ text: formatCount(f.entries) }),
};

/** Total of a longer Period, against the comparison scaled to the Period's length. */
export const TOTAL: Column = {
  id: "total",
  label: "Total",
  tooltip: "All the time in this group",
  cell: (f) => span(f.total, true),
};

/** Total of a one-Day Report: against the Usual it moves by the Usual's average per Day. */
export const DAY_TOTAL: Column = {
  id: "day-total",
  label: "Total",
  tooltip: "All the time in this group on this Day",
  cell: (f) => span(f.dayTotal, true),
};

/** The Usual's average per Day. */
export const USUAL: Column = {
  id: "usual",
  label: "Usual",
  tooltip: `How much time this group usually gets on a Day it happens, over the ${USUAL_SPAN}`,
  cell: (f) => ({ text: f.usual ? formatSpan(f.usual.avg) : NONE, change: null }),
};

/** The Usual's smallest to largest daily total. */
export const USUAL_RANGE: Column = {
  id: "usual-range",
  label: "Usual range",
  tooltip: `The smallest to the largest daily total of this group over the ${USUAL_SPAN}`,
  cell: (f) => ({ text: f.usual ? formatSpanRange(f.usual.min, f.usual.max) : NONE }),
};

const SPREAD_WORDS: Record<StatMode, { unit: string; avg: string; min: string; max: string }> = {
  per_day: {
    unit: "per Day",
    avg: "The average daily total over the Days this group happened",
    min: "The smallest daily total among the Days this group happened",
    max: "The largest daily total among the Days this group happened",
  },
  per_entry: {
    unit: "per entry",
    avg: "The average length of a Time Entry in this group",
    min: "The shortest Time Entry in this group",
    max: "The longest Time Entry in this group",
  },
};

/** Avg, Min and Max in the chosen stat mode. */
export function spreadColumns(mode: StatMode): Column[] {
  const words = SPREAD_WORDS[mode];
  return [
    { id: "avg", label: `Avg ${words.unit}`, tooltip: words.avg, cell: (f) => span(f.avg) },
    { id: "min", label: `Min ${words.unit}`, tooltip: words.min, cell: (f) => span(f.min) },
    { id: "max", label: `Max ${words.unit}`, tooltip: words.max, cell: (f) => span(f.max) },
  ];
}
