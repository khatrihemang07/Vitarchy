import { useNavigate, useSearch } from "@tanstack/react-router";
import { useCallback, useMemo } from "react";
import { parseTimelineSearch, type TimelineSearch, timelineParams } from "./timelineSearch";

/** The page state kept in the URL, with a setter that writes it back (only non-default values are kept). */
export function useTimelineSearch() {
  const raw: Record<string, unknown> = useSearch({ strict: false });
  const search = useMemo(() => parseTimelineSearch(raw), [raw]);
  const navigate = useNavigate();
  const update = useCallback(
    (next: TimelineSearch) => {
      void navigate({ to: "/time/timeline", search: timelineParams(next) });
    },
    [navigate],
  );
  return { search, update };
}
