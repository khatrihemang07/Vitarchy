import type { components } from "../schema";

/** What `GET /api/time/status` returns. */
export type TimeStatus = components["schemas"]["TimeStatus"];
/** One Sync or Import attempt. */
export type TimeRun = components["schemas"]["RunInfo"];
/** What a Sync did. */
export type SyncCounts = components["schemas"]["SyncCounts"];

/** What an Import did. */
export type ImportCounts = components["schemas"]["ImportCounts"];

/** Imported Activity, the Snapshot and the latest Import. */
export type ActivityStatus = components["schemas"]["ActivityStatus"];
