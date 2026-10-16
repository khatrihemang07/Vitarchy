import type { TimeRun } from "@/api/time";
import { describeRun } from "./describeRun";

const counts = (values: Record<string, number>) => values as unknown as TimeRun["counts"];

const run = (over: Partial<TimeRun>): TimeRun => ({
  kind: "sync",
  started_at: "2026-04-10T12:00:00Z",
  finished_at: "2026-04-10T12:00:03Z",
  outcome: "success",
  counts: {},
  error_code: null,
  ...over,
});

describe("describeRun", () => {
  it("lists the non-zero counts", () => {
    const shown = counts({ inserted: 12, updated: 3, deleted: 0, skipped_running: 1 });
    expect(describeRun(run({ counts: shown }))).toBe("12 new, 3 updated, 1 still running");
  });

  it("counts an Import's new, updated and skipped rows but not the unchanged ones", () => {
    const shown = counts({ read: 20, inserted: 5, updated: 1, unchanged: 12, skipped: 2 });
    expect(describeRun(run({ kind: "import", counts: shown }))).toBe("5 new, 1 updated, 2 skipped");
  });

  it("says when nothing changed", () => {
    expect(describeRun(run({ counts: counts({ inserted: 0, updated: 0 }) }))).toBe("Nothing to bring in.");
  });

  it("explains a failed run by its error code", () => {
    expect(describeRun(run({ outcome: "failed", error_code: "invalid_token" }))).toMatch(/rejected this API token/);
  });

  it("explains a failed Import by its error code", () => {
    expect(describeRun(run({ kind: "import", outcome: "failed", error_code: "no_snapshot" }))).toMatch(/No Snapshot yet/);
  });

  it("has a fallback for a failure it does not know", () => {
    expect(describeRun(run({ outcome: "failed", error_code: "database" }))).toBe("The last run failed.");
  });
});
