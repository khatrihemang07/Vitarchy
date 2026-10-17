import { keepPreviousData, useQuery } from "@tanstack/react-query";
import { createLogger } from "@/lib/log";
import { callApi } from "../callApi";
import { api } from "../client";
import { timeReportKey } from "./keys";
import type { Report, ReportQuery } from "./types";

const log = createLogger("time");

/**
 * Loads the Report for one Period. Counts are logged, never the search text or
 * descriptions. Repeated filters (`project=1&project=2`) are openapi-fetch's
 * default array encoding, which the server reads as lists.
 */
export async function fetchReport(query: ReportQuery): Promise<Report> {
  const report = await callApi(() => api.GET("/api/time/report", { params: { query } }));
  log.debug("report loaded", {
    kind: query.kind,
    groups: report.groups.length,
    projects: query.project?.length ?? 0,
    tags: query.tag?.length ?? 0,
    clients: query.client?.length ?? 0,
    searched: Boolean(query.q),
  });
  return report;
}

/**
 * Query for a Report. The previous Report stays on screen while the next Period
 * loads, so moving between Periods does not flash an empty page.
 */
export function useReport(query: ReportQuery | null) {
  return useQuery({
    queryKey: timeReportKey(query),
    queryFn: () => fetchReport(query as ReportQuery),
    enabled: query !== null,
    placeholderData: keepPreviousData,
    retry: false,
  });
}
