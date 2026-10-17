import { useFacets, useReport, useTimeStatus } from "@/api/time";
import { Link } from "@tanstack/react-router";
import { Button, FadeUp, stagger } from "@/design";
import { errorMessage } from "../failureMessage";
import { TimeNotice } from "../TimeNotice";
import { useClock } from "../useClock";
import { CompareToggle } from "./CompareToggle";
import { compareOf } from "./compareMode";
import { FilterBar } from "./FilterBar";
import { GroupBySelect } from "./GroupBySelect";
import { PeriodPicker } from "./PeriodPicker";
import { containsDay, todayOf } from "./period";
import { ReportBody } from "./ReportBody";
import { buildReportQuery } from "./reportQuery";
import { hasAnyFilter } from "./reportActions";
import { rangeOf } from "./reportSearch";
import { StatModeToggle } from "./StatModeToggle";
import { useDisplaySettings } from "./useDisplaySettings";
import { useReportHandlers } from "./useReportHandlers";
import { useReportSearch } from "./useReportSearch";
import { useStartingFilters } from "./useStartingFilters";

/** Header and tabs are 0 and 1, the filters 2; the report arrives after them. */
const FILTERS_STAGGER = 2;
const BODY_STAGGER = FILTERS_STAGGER + 1;

const browserTimeZone = () => Intl.DateTimeFormat().resolvedOptions().timeZone;

/** `/time/reports`: pick a Period and filters, group the Time Entries, and compare with the Previous Period or the last 4 weeks. */
export function ReportsPage() {
  const { search: urlSearch, update } = useReportSearch();
  const settings = useDisplaySettings();
  const facets = useFacets();
  const status = useTimeStatus();
  const search = useStartingFilters(urlSearch, facets.data, update);
  const today = todayOf(useClock());
  const compare = compareOf(search);
  const range = rangeOf(search, today, settings.weekStart);
  const handlers = useReportHandlers({ search, range, today, weekStart: settings.weekStart, update });

  const filtersSettled = facets.isSuccess || facets.isError;
  const report = useReport(settings.ready && filtersSettled ? buildReportQuery(search, range, browserTimeZone()) : null);
  const neverSynced = status.data?.connected === false && status.data.time_entry_count === 0;

  const showReport = !neverSynced;

  return (
    <div className="flex flex-col gap-6">
      {settings.failed ? (
        <TimeNotice kind="settings-fallback" title="Using the default week start" onRetry={settings.retry}>
          Settings could not load.
        </TimeNotice>
      ) : null}
      <div className="animate-fade-up flex flex-col gap-3" style={stagger(FILTERS_STAGGER)}>
        <PeriodPicker kind={search.kind} range={range} isCurrent={search.kind !== "custom" && containsDay(range, today)} onKind={handlers.onKind} onShift={handlers.onShift} onToday={handlers.onToday} onCustom={handlers.onCustom} />
        <FilterBar facets={facets.data} search={search} active={hasAnyFilter(search)} onFacet={handlers.onFacet} onSearch={handlers.onSearch} onClear={handlers.onClear} />
        <div className="flex flex-wrap items-center justify-between gap-3">
          <GroupBySelect group1={search.group1} group2={search.group2} onChange={handlers.onGroups} />
          <div className="flex flex-wrap items-center gap-3">
            {search.kind === "day" ? null : <StatModeToggle mode={search.mode} onChange={handlers.onMode} />}
            <CompareToggle compare={compare} onChange={handlers.onCompare} />
          </div>
        </div>
      </div>
      {neverSynced ? (
        <FadeUp index={BODY_STAGGER}>
          <TimeNotice
            kind="not-connected"
            title="Toggl is not connected yet"
            action={
              <Button asChild variant="link">
                <Link to="/time">Connect Toggl and Sync</Link>
              </Button>
            }
          >
            Reports are built from your Time Entries. Connect Toggl and Sync to see them here.
          </TimeNotice>
        </FadeUp>
      ) : null}
      {showReport && report.isPending ? (
        <FadeUp index={BODY_STAGGER}>
          <TimeNotice kind="loading" title="Loading report…" />
        </FadeUp>
      ) : null}
      {showReport && report.isError ? (
        <FadeUp index={BODY_STAGGER}>
          <TimeNotice kind="failure" title={errorMessage(report.error)} onRetry={() => void report.refetch()} />
        </FadeUp>
      ) : null}
      {showReport && report.data ? (
        <div aria-busy={report.isPlaceholderData} className={report.isPlaceholderData ? "opacity-60 transition-opacity" : "transition-opacity"}>
          <ReportBody report={report.data} mode={search.mode} compare={compare} clock={settings.clock} filtered={hasAnyFilter(search)} onClear={handlers.onClear} firstStagger={BODY_STAGGER} />
        </div>
      ) : null}
    </div>
  );
}
