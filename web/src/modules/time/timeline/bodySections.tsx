import type { ReactNode } from "react";
import type { TimelineSummary } from "@/api/time";
import type { Clock } from "../reports/entryTimes";
import { appColorMap } from "./appColors";
import { AppsSection } from "./AppsSection";
import { DaySection } from "./DaySection";
import { SegmentList } from "./SegmentList";
import { ShapeSection } from "./ShapeSection";
import { TimelineDays } from "./TimelineDays";
import { TimelineOverview } from "./TimelineOverview";
import { TimelineSection } from "./TimelineSection";

/** One block of the page with the key React tracks it by. */
export interface BodySection {
  key: string;
  node: ReactNode;
}

interface SectionInput {
  summary: TimelineSummary;
  clock: Clock;
  now: Date;
  highlight?: string | undefined;
  onApp: (app: string | undefined) => void;
}

/** The sections of a loaded Timeline in page order: a Day has its band and Segments, a longer Period its per-Day bars. */
export function bodySections({ summary, clock, now, highlight, onApp }: SectionInput): BodySection[] {
  const colors = appColorMap(summary.apps);
  const isDay = summary.period.kind === "day";
  const sections: BodySection[] = [{ key: "overview", node: <TimelineOverview summary={summary} /> }];
  sections.push(
    isDay
      ? { key: "day", node: <DaySection summary={summary} colors={colors} clock={clock} now={now} highlight={highlight} onApp={onApp} /> }
      : { key: "days", node: <TimelineSection title="Per Day"><TimelineDays days={summary.days} /></TimelineSection> },
    { key: "shape", node: <ShapeSection summary={summary} clock={clock} isDay={isDay} /> },
    { key: "apps", node: <AppsSection summary={summary} colors={colors} highlight={highlight} onApp={onApp} /> },
  );
  if (isDay) {
    sections.push({
      key: "segments",
      node: <TimelineSection title="Segments" flush><SegmentList segments={summary.segments ?? []} colors={colors} clock={clock} highlight={highlight} /></TimelineSection>,
    });
  }
  return sections;
}
