/** Query key of the Time status; the one place it is written. */
export const timeStatusKey = ["time", "status"] as const;

/** Query key of a Report; the query is part of the key so each Period and filter set caches separately. */
export const timeReportKey = (query: unknown) => ["time", "report", query] as const;

/** Query key of the filter choices. */
export const timeFacetsKey = ["time", "facets"] as const;

/** Query key of the Away rules preview. */
export const timeAwayPreviewKey = ["time", "away-preview"] as const;

/** Query key of a Timeline summary; the query is part of the key so each Period caches separately. */
export const timeTimelineKey = (query: unknown) => ["time", "timeline", query] as const;

/** Prefix of every Timeline summary key, for invalidating them all after the Timeline changes. */
export const timeTimelineKeyPrefix = ["time", "timeline"] as const;
