import { useTimelineSummary } from "@/api/time";
import { FadeUp, Notice, PeriodPicker, QueryNotices, SectionCard, stagger, TableSkeleton } from "@/design";
import { browserTimeZone } from "../browserTimeZone";
import { errorMessage } from "../failureMessage";
import { containsDay, todayOf } from "@/lib/period/period";
import { rangeOf } from "@/lib/period/periodSearch";
import { useDisplaySettings } from "../reports/useDisplaySettings";
import { useClock } from "@/lib/useClock";
import { TimelineBody } from "./TimelineBody";
import { useTimelineHandlers } from "./useTimelineHandlers";
import { useTimelineSearch } from "./useTimelineSearch";
import { TIME_NOTICE_VIEWS } from "../noticeViews";

/** The picker is 2 in the page's entrance sequence (header and tabs are 0 and 1); the body follows. */
const PICKER_STAGGER = 2;
const BODY_STAGGER = PICKER_STAGGER + 1;


/** `/time/timeline`: pick a Period and see what you did on this computer, app by app, against the Usual. */
export function TimelinePage() {
  const { search, update } = useTimelineSearch();
  const settings = useDisplaySettings();
  const now = useClock();
  const today = todayOf(now);
  const range = rangeOf(search, today, settings.weekStart);
  const handlers = useTimelineHandlers({ search, range, today, weekStart: settings.weekStart, update });
  const summary = useTimelineSummary(settings.ready ? { kind: search.kind, from: range.from, to: range.to, tz: browserTimeZone() } : null);

  return (
    <div className="flex flex-col gap-6">
      {settings.failed ? (
        <Notice kind="settings-fallback" views={TIME_NOTICE_VIEWS} title="Using the default week start" onRetry={settings.retry}>
          Settings could not load.
        </Notice>
      ) : null}
      <div className="animate-fade-up flex flex-col gap-3" style={stagger(PICKER_STAGGER)}>
        <PeriodPicker kind={search.kind} range={range} isCurrent={search.kind !== "custom" && containsDay(range, today)} onKind={handlers.onKind} onShift={handlers.onShift} onToday={handlers.onToday} onCustom={handlers.onCustom} />
      </div>
      {summary.isPending || summary.isError ? (
        <FadeUp index={BODY_STAGGER}>
          <QueryNotices
            query={summary}
            describeError={errorMessage}
            skeleton={<SectionCard title="Timeline"><TableSkeleton label="Loading Timeline" rows={6} columns={3} /></SectionCard>}
          />
        </FadeUp>
      ) : null}
      {summary.data ? (
        <div aria-busy={summary.isPlaceholderData} className={summary.isPlaceholderData ? "opacity-60 transition-opacity" : "transition-opacity"}>
          <TimelineBody summary={summary.data} clock={settings.clock} now={now} highlight={search.app} onApp={handlers.onApp} firstStagger={BODY_STAGGER} />
        </div>
      ) : null}
    </div>
  );
}
