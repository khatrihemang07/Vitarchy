import { getRouteApi } from "@tanstack/react-router";
import { useCoreSettings } from "@/app/module-manifest";
import { localIsoDate, type PeriodSelection, type WeekStart } from "./period";
import type { TimeTabId } from "./tabs";
import { useClock } from "./useClock";
import { parseTimeSearch, resolveView, toSearch, type TimeSearch, type TimeView } from "./timeSearch";

const route = getRouteApi("/time");

/** The app's route tree is not registered for Module code, so the router types search params as `never`. */
const asSearch = (search: TimeSearch) => search as never;

/** The Time page's tab and Period, read from and written to the URL so a view can be bookmarked. */
export function useTimeView(): TimeView & {
  today: string;
  weekStart: WeekStart;
  setTab: (tab: TimeTabId) => void;
  setSelection: (selection: PeriodSelection) => void;
} {
  // Parsed again: the router merges unvalidated parent params back in, so bad ones can reappear.
  const search = parseTimeSearch(route.useSearch());
  const navigate = route.useNavigate();
  const weekStart = useCoreSettings().week_start;
  // The clock ticks, so "today" rolls over past local midnight while the page stays open.
  const today = localIsoDate(useClock());
  const view = resolveView(search, today, weekStart);
  return {
    ...view,
    today,
    weekStart,
    setTab: (tab) => void navigate({ search: asSearch(toSearch(tab, view.selection)) }),
    setSelection: (selection) => void navigate({ search: asSearch(toSearch(view.tab, selection)) }),
  };
}
