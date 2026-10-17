import { toTimeFailure, useFacets, useReport, useTimeStatus } from "@/api/time";
import { FailureNotice } from "../FailureNotice";
import { LoadingNotice } from "../LoadingNotice";
import { TimeNav } from "../TimeNav";
import { useClock } from "../useClock";
import { FilterBar } from "./FilterBar";
import { GroupBySelect } from "./GroupBySelect";
import { NotConnectedNotice } from "./NotConnectedNotice";
import { PeriodPicker } from "./PeriodPicker";
import { containsDay, todayOf } from "./period";
import { ReportBody } from "./ReportBody";
import { buildReportQuery } from "./reportQuery";
import { hasAnyFilter } from "./reportActions";
import { rangeOf } from "./reportSearch";
import { SettingsFallbackNotice } from "./SettingsFallbackNotice";
import { StatModeToggle } from "./StatModeToggle";
import { useDisplaySettings } from "./useDisplaySettings";
import { useReportHandlers } from "./useReportHandlers";
import { useReportSearch } from "./useReportSearch";
import { useStartingFilters } from "./useStartingFilters";

const browserTimeZone = () => Intl.DateTimeFormat().resolvedOptions().timeZone;

/** `/time/reports`: pick a Period and filters, group the Time Entries, and compare with the Previous Period. */
export function ReportsPage() {
  const { search: urlSearch, update } = useReportSearch();
  const settings = useDisplaySettings();
  const facets = useFacets();
  const status = useTimeStatus();
  const search = useStartingFilters(urlSearch, facets.data, update);
  const today = todayOf(useClock());
  const range = rangeOf(search, today, settings.weekStart);
  const handlers = useReportHandlers({ search, range, today, weekStart: settings.weekStart, update });

  const filtersSettled = facets.isSuccess || facets.isError;
  const report = useReport(settings.ready && filtersSettled ? buildReportQuery(search, range, browserTimeZone()) : null);
  const neverSynced = status.data?.connected === false && status.data.time_entry_count === 0;

  return (
    <div className="flex flex-col gap-5">
      <TimeNav />
      <header>
        <h1 className="font-display text-4xl font-bold tracking-tight">Reports</h1>
        <p className="text-muted-foreground">Where your Time Entries went, compared with the Previous Period.</p>
      </header>
      {settings.failed ? <SettingsFallbackNotice onRetry={settings.retry} /> : null}
      <div className="flex flex-col gap-3">
        <PeriodPicker kind={search.kind} range={range} isCurrent={search.kind !== "custom" && containsDay(range, today)} onKind={handlers.onKind} onShift={handlers.onShift} onToday={handlers.onToday} onCustom={handlers.onCustom} />
        <FilterBar facets={facets.data} search={search} active={hasAnyFilter(search)} onFacet={handlers.onFacet} onSearch={handlers.onSearch} onClear={handlers.onClear} />
        <div className="flex flex-wrap items-center justify-between gap-3">
          <GroupBySelect group1={search.group1} group2={search.group2} onChange={handlers.onGroups} />
          <StatModeToggle mode={search.mode} onChange={handlers.onMode} />
        </div>
      </div>
      {neverSynced ? <NotConnectedNotice /> : null}
      {!neverSynced && report.isPending ? <LoadingNotice>Loading report…</LoadingNotice> : null}
      {!neverSynced && report.isError ? <FailureNotice failure={toTimeFailure(report.error)} /> : null}
      {!neverSynced && report.data ? (
        <div aria-busy={report.isPlaceholderData} className={report.isPlaceholderData ? "opacity-60 transition-opacity" : "transition-opacity"}>
          <ReportBody report={report.data} mode={search.mode} clock={settings.clock} today={today} filtered={hasAnyFilter(search)} onClear={handlers.onClear} />
        </div>
      ) : null}
    </div>
  );
}
