import { type ComponentType, type LazyExoticComponent, lazy } from "react";
import type { PeriodSelection } from "./period";
import type { TimeTabId } from "./tabs";

/** What a tab body that shows a Period is given. */
export interface PeriodBodyProps {
  selection: PeriodSelection;
  /** Switches to a tab showing a Period (one navigation, so Back returns to where the person was). */
  open: (tab: TimeTabId, selection: PeriodSelection) => void;
}

/**
 * Tab bodies loaded on first visit, so a chart library is not in the first page load.
 * A tab not listed here shows its neutral note (or, for Sources, its own cards).
 */
export const LAZY_BODIES: Partial<Record<TimeTabId, LazyExoticComponent<ComponentType<PeriodBodyProps>>>> = {
  overview: lazy(() => import("./overview/OverviewTab")),
  activity: lazy(() => import("./activity/ActivityTab")),
  "life-log": lazy(() => import("./lifelog/LifeLogTab")),
};
