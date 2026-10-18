import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook } from "@testing-library/react";
import type { ReactNode } from "react";
import { timeTimelineKey } from "./keys";
import { useSyncTime } from "./sync";

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("useSyncTime", () => {
  it("marks every cached Timeline stale, since the Day lane shows Time Entries", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(() =>
        Promise.resolve(
          new Response(JSON.stringify({ upserted: 0, deleted: 0 }), { status: 200, headers: { "content-type": "application/json" } }),
        ),
      ),
    );
    const client = new QueryClient({ defaultOptions: { mutations: { retry: false } } });
    client.setQueryData(timeTimelineKey({ from: "2026-10-17" }), { cached: true });
    const wrapper = ({ children }: { children: ReactNode }) => <QueryClientProvider client={client}>{children}</QueryClientProvider>;
    const { result } = renderHook(() => useSyncTime(), { wrapper });

    await act(async () => {
      await result.current.mutateAsync().catch(() => undefined);
    });

    expect(client.getQueryState(timeTimelineKey({ from: "2026-10-17" }))?.isInvalidated).toBe(true);
  });
});
