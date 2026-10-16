import { CalendarDays, Clock, type LucideIcon, Timer, TrendingUp, Zap } from "lucide-react";
import type { InsightKind } from "@/api/time";

/** One icon per kind of Insight, so the list can be scanned by shape as well as by words. */
export const INSIGHT_ICONS: Record<InsightKind, LucideIcon> = {
  typical_time: Clock,
  typical_duration: Timer,
  weekday_difference: CalendarDays,
  change: TrendingUp,
  outlier: Zap,
};
