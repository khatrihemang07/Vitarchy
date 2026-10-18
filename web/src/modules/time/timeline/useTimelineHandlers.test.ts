import { renderHook } from "@testing-library/react";
import type { TimelineSearch } from "./timelineSearch";
import { useTimelineHandlers } from "./useTimelineHandlers";

const range = { from: "2026-10-12", to: "2026-10-18" };

function setup(kind: "week" | "day" = "week") {
  const update = vi.fn<(next: TimelineSearch) => void>();
  const search = { kind, from: range.from, to: range.to, app: "Code" } as const;
  const { result } = renderHook(() => useTimelineHandlers({ search, range, today: "2026-10-14", weekStart: "monday", update }));
  return { handlers: result.current, update };
}

describe("useTimelineHandlers", () => {
  it("drops the highlight and keeps the kind when shifting the Period", () => {
    const { handlers, update } = setup();
    handlers.onShift(1);
    const next = update.mock.calls[0]?.[0];
    expect(next).toMatchObject({ kind: "week" });
    expect(next).not.toHaveProperty("app");
  });

  it("drops the highlight and keeps the kind when jumping to today", () => {
    const { handlers, update } = setup("day");
    handlers.onToday();
    const next = update.mock.calls[0]?.[0];
    expect(next).toMatchObject({ kind: "day" });
    expect(next).not.toHaveProperty("app");
  });

  it("highlights an app, keeping the Period", () => {
    const { handlers, update } = setup();
    handlers.onApp("Safari");
    expect(update).toHaveBeenCalledWith(expect.objectContaining({ kind: "week", app: "Safari" }));
  });
});
