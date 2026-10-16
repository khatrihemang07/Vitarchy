/** Query key of the Time status; the one place it is written. */
export const timeStatusKey = ["time", "status"] as const;

/** Query key of one Activity analysis for a Period, so a Period change refetches it. */
export const activityKey = (analysis: string, query: object) => ["time", "analysis", "activity", analysis, query] as const;
