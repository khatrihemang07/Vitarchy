import type { TimelineSummary } from "@/api/time";
import type { AppColors } from "./appColors";
import { AppList } from "./AppList";
import { HighlightNotice } from "./HighlightNotice";
import { TimelineSection } from "./TimelineSection";

interface AppsSectionProps {
  summary: TimelineSummary;
  colors: AppColors;
  highlight?: string | undefined;
  onApp: (app: string | undefined) => void;
}

/** The ranked apps, with a notice (and a way to clear it) while one is highlighted. */
export function AppsSection({ summary, colors, highlight, onApp }: AppsSectionProps) {
  return (
    <div className="flex flex-col gap-3">
      {highlight ? <HighlightNotice app={highlight} onClear={() => { onApp(undefined); }} /> : null}
      <TimelineSection title="Apps" flush><AppList apps={summary.apps} colors={colors} highlight={highlight} onApp={onApp} /></TimelineSection>
    </div>
  );
}
