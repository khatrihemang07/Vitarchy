import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { jsonResponse, stubApi } from "../test-utils";
import { useDisplaySettings } from "./useDisplaySettings";

afterEach(() => {
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
});

function wrapper({ children }: { children: ReactNode }) {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
}

describe("useDisplaySettings", () => {
  it("reads the week start and clock from Core settings", async () => {
    stubApi({ "GET /api/settings": () => jsonResponse(200, { core: { week_start: "sunday", clock: "h12" }, modules: {} }) });
    const { result } = renderHook(useDisplaySettings, { wrapper });
    await waitFor(() => { expect(result.current.ready).toBe(true); });
    expect(result.current).toMatchObject({ weekStart: "sunday", clock: "h12" });
  });

  it("falls back to the default week start and logs a warning when Core settings fail to load", async () => {
    const warn = vi.spyOn(console, "warn").mockImplementation(() => undefined);
    stubApi({ "GET /api/settings": () => jsonResponse(500, { title: "boom" }) });
    const { result } = renderHook(useDisplaySettings, { wrapper });
    await waitFor(() => { expect(result.current.ready).toBe(true); });
    expect(result.current.weekStart).toBe("monday");
    await waitFor(() => { expect(warn).toHaveBeenCalled(); });
  });
});
