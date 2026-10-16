import { act, renderHook } from "@testing-library/react";
import { useDismissedSuggestions } from "./dismissedSuggestions";

afterEach(() => { sessionStorage.clear(); });

describe("useDismissedSuggestions", () => {
  it("keeps every suggestion dismissed in the same moment, in memory and in the session", () => {
    const { result } = renderHook(() => useDismissedSuggestions());
    act(() => {
      result.current.dismiss("a");
      result.current.dismiss("b");
    });
    expect([...result.current.dismissed].sort()).toEqual(["a", "b"]);
    expect((JSON.parse(sessionStorage.getItem("vitarchy.time.dismissedRoutineSuggestions") ?? "[]") as string[]).sort()).toEqual(["a", "b"]);
  });

  it("starts from what the session already holds", () => {
    sessionStorage.setItem("vitarchy.time.dismissedRoutineSuggestions", JSON.stringify(["old"]));
    const { result } = renderHook(() => useDismissedSuggestions());
    expect(result.current.dismissed.has("old")).toBe(true);
  });
});
