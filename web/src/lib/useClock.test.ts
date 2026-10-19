import { act, renderHook } from "@testing-library/react";
import { useClock } from "./useClock";

describe("useClock", () => {
  afterEach(() => vi.useRealTimers());

  it("moves on every 30 seconds so relative times stay true", () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-10-19T10:00:00Z"));
    const { result } = renderHook(() => useClock());
    const first = result.current.getTime();
    act(() => { vi.advanceTimersByTime(30_000); });
    expect(result.current.getTime()).toBe(first + 30_000);
  });
});
