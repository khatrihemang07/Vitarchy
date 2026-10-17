import { useNavigate, useSearch } from "@tanstack/react-router";
import { useCallback, useMemo } from "react";
import { saveLastFilters } from "./lastFilters";
import { parseReportSearch, type ReportSearch, toSearchParams } from "./reportSearch";

/** The page state kept in the URL, with a setter that also remembers the filters for next time. */
export function useReportSearch() {
  const raw: Record<string, unknown> = useSearch({ strict: false });
  const search = useMemo(() => parseReportSearch(raw), [raw]);
  const navigate = useNavigate();
  const update = useCallback(
    (next: ReportSearch, options: { replace?: boolean } = {}) => {
      saveLastFilters(next);
      void navigate({ to: "/time/reports", search: toSearchParams(next), replace: options.replace ?? false });
    },
    [navigate],
  );
  return { search, update };
}
