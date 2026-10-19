import { act, renderHook } from "@testing-library/react";
import { useDelayedFlag } from "./useDelayedFlag";

describe("useDelayedFlag", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });
  afterEach(() => {
    vi.useRealTimers();
  });

  it("stays false until the delay has passed", () => {
    const { result } = renderHook(() => useDelayedFlag(true, 250));
    expect(result.current).toBe(false);
    act(() => { vi.advanceTimersByTime(249); });
    expect(result.current).toBe(false);
    act(() => { vi.advanceTimersByTime(1); });
    expect(result.current).toBe(true);
  });

  it("never turns true when the work finishes inside the delay", () => {
    const { result, rerender } = renderHook(({ active }) => useDelayedFlag(active, 250), { initialProps: { active: true } });
    act(() => { vi.advanceTimersByTime(200); });
    rerender({ active: false });
    act(() => { vi.advanceTimersByTime(1000); });
    expect(result.current).toBe(false);
  });

  it("turns false at once when the work finishes after the delay", () => {
    const { result, rerender } = renderHook(({ active }) => useDelayedFlag(active, 250), { initialProps: { active: true } });
    act(() => { vi.advanceTimersByTime(300); });
    expect(result.current).toBe(true);
    rerender({ active: false });
    expect(result.current).toBe(false);
  });

  it("starts the delay again for a second stretch of work", () => {
    const { result, rerender } = renderHook(({ active }) => useDelayedFlag(active, 250), { initialProps: { active: true } });
    act(() => { vi.advanceTimersByTime(300); });
    rerender({ active: false });
    rerender({ active: true });
    expect(result.current).toBe(false);
    act(() => { vi.advanceTimersByTime(250); });
    expect(result.current).toBe(true);
  });

  it("is true straight away when there is no delay", () => {
    const { result } = renderHook(() => useDelayedFlag(true, 0));
    expect(result.current).toBe(true);
  });
});
