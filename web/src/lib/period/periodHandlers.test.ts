import { periodHandlers } from "./periodHandlers";

const october = { from: "2026-10-01", to: "2026-10-31" };

function setup(extra: Record<string, unknown> = {}) {
  const update = vi.fn();
  const handlers = periodHandlers({ search: { kind: "month" as const, ...extra }, range: october, today: "2026-10-19", weekStart: "monday", update });
  return { update, handlers };
}

describe("periodHandlers", () => {
  it("moves to the previous and next Period, keeping the rest of the state", () => {
    const { update, handlers } = setup({ q: "rent" });
    handlers.onShift(-1);
    handlers.onShift(1);
    expect(update.mock.calls).toEqual([
      [{ kind: "month", date: "2026-09-01", q: "rent" }],
      [{ kind: "month", date: "2026-11-01", q: "rent" }],
    ]);
  });

  it("changes kind around today, and to a custom stretch", () => {
    const { update, handlers } = setup();
    handlers.onKind("week");
    handlers.onCustom({ from: "2026-10-05", to: "2026-10-02" });
    expect(update.mock.calls).toEqual([
      [{ kind: "week", date: "2026-10-19" }],
      [{ kind: "custom", from: "2026-10-02", to: "2026-10-05" }],
    ]);
  });

  it("jumps to the Period that holds today", () => {
    const { update, handlers } = setup();
    handlers.onToday();
    expect(update).toHaveBeenCalledWith({ kind: "month", date: "2026-10-01" });
  });
});
