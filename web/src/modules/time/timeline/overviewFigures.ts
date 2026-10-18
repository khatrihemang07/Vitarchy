import type { TimelineSummary, UsualFigure } from "@/api/time";
import { formatCount } from "../format";
import type { ChangeUnit } from "../reports/formatChange";
import { periodLabel } from "../reports/period";
import { USUAL_PHRASE } from "../usualWords";

/** One headline tile: its value, how it is written, and its change from the Usual (null when not comparable). */
export interface OverviewFigure {
  label: string;
  value: number;
  unit: ChangeUnit;
  change: number | null;
  /** What the change is measured against, when that is not the Usual's total; absent without a Usual. */
  note?: string;
  /** A link under the tile, for the figure whose rule is explained elsewhere. */
  link?: { label: string; to: string };
}

type Usual = NonNullable<TimelineSummary["usual"]>;

/** The change of a total: current minus the Usual's total scaled to the Days that have passed. */
const totalChange = (value: number, figure: UsualFigure | undefined): number | null =>
  figure?.scaled_total == null ? null : value - figure.scaled_total;

/**
 * The longest Focus stretch does not add up across Days, so it is compared per Day: with the
 * Usual's average Day for a Day, and with its best Day for a longer Period (whose figure is
 * itself the best stretch of many Days).
 */
function focusFigure(summary: TimelineSummary, usual: Usual | null | undefined): Pick<OverviewFigure, "change" | "note"> {
  if (!usual) return { change: null };
  const spread = usual.longest_focus.per_day;
  const isDay = summary.period.kind === "day";
  return {
    change: summary.overview.longest_focus_secs - (isDay ? spread.avg_s : spread.max_s),
    note: isDay ? "vs your usual Day" : `vs your best Day in ${USUAL_PHRASE}`,
  };
}

/** Where the Away rules are shown: the Sources page. */
const AWAY_RULES_LINK = { label: "How Away is decided", to: "/time" } as const;

/** The five tiles of the overview, in order. */
export function overviewFigures(summary: TimelineSummary): OverviewFigure[] {
  const { overview, usual } = summary;
  return [
    { label: "Active", value: overview.active_secs, unit: "seconds", change: totalChange(overview.active_secs, usual?.active) },
    { label: "Away", value: overview.away_secs, unit: "seconds", change: totalChange(overview.away_secs, usual?.away), link: AWAY_RULES_LINK },
    { label: "Away unknown", value: overview.unknown_secs, unit: "seconds", change: totalChange(overview.unknown_secs, usual?.unknown) },
    { label: "Switches", value: overview.switches, unit: "count", change: totalChange(overview.switches, usual?.switches) },
    { label: "Longest focus", value: overview.longest_focus_secs, unit: "seconds", ...focusFigure(summary, usual) },
  ];
}

/** The sentence under the tiles saying what the changes are measured against. */
export function usualNote({ period, usual }: TimelineSummary): string {
  if (!usual) return `Nothing to compare with yet: ${USUAL_PHRASE} has no Timeline.`;
  const dates = periodLabel("custom", { from: period.usual_from, to: period.usual_to });
  const days = `${formatCount(usual.days)} ${usual.days === 1 ? "Day" : "Days"} with Timeline`;
  return `Compared with ${USUAL_PHRASE}, ${dates} (${days})`;
}

/** True when the Period has any Timeline at all (otherwise the page offers to Import Activity). */
export function hasTimeline({ overview }: TimelineSummary): boolean {
  return overview.active_secs + overview.away_secs + overview.unknown_secs > 0;
}
