import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { jsonResponse, stubApi } from "../test-utils";
import { useWeekStart } from "./useWeekStart";

afterEach(() => vi.unstubAllGlobals());

const wrapper = ({ children }: { children: ReactNode }) => (
  <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}>{children}</QueryClientProvider>
);

describe("useWeekStart", () => {
  it("is not ready until Settings answer, then follows the Core setting", async () => {
    stubApi({ "GET /api/settings": () => jsonResponse(200, { core: { week_start: "sunday" } }) });
    const { result } = renderHook(() => useWeekStart(), { wrapper });
    expect(result.current.ready).toBe(false);
    await waitFor(() => { expect(result.current.ready).toBe(true); });
    expect(result.current.weekStart).toBe("sunday");
  });

  it("is ready with the default week start when Settings cannot load", async () => {
    stubApi({ "GET /api/settings": () => jsonResponse(500, {}) });
    const { result } = renderHook(() => useWeekStart(), { wrapper });
    await waitFor(() => { expect(result.current.ready).toBe(true); });
    expect(result.current.weekStart).toBe("monday");
  });
});
