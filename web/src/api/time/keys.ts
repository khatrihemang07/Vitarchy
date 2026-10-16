/** Query key of the Time status; the one place it is written. */
export const timeStatusKey = ["time", "status"] as const;

/** Query key of one Activity analysis for a Period, so a Period change refetches it. */
export const activityKey = (analysis: string, query: object) => ["time", "analysis", "activity", analysis, query] as const;

/** Query key of one Life log analysis for a Period, so a Period change refetches it. */
export const lifeLogKey = (analysis: string, query: object) => ["time", "analysis", "lifelog", analysis, query] as const;

/** Query key of anything about Routines; invalidating this prefix refreshes every Routine query. */
export const routinesKey = (...parts: readonly unknown[]) => ["time", "routines", ...parts] as const;

/** Query key of the Insights for a Period, so a Period change refetches them. */
export const insightsKey = (query: object) => ["time", "analysis", "insights", query] as const;
