import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { useSetConnection } from "./connection";

afterEach(() => {
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
});

function wrapper({ children }: { children: ReactNode }) {
  const client = new QueryClient({ defaultOptions: { mutations: { retry: false } } });
  return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
}

describe("useSetConnection", () => {
  it("logs why a connect failed by its failure kind, never the token", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(() =>
        Promise.resolve(
          new Response(JSON.stringify({ status: 401, detail: "rejected", code: "invalid_token" }), {
            status: 401,
            headers: { "content-type": "application/problem+json" },
          }),
        ),
      ),
    );
    const warn = vi.spyOn(console, "warn").mockImplementation(() => undefined);
    const { result } = renderHook(() => useSetConnection(), { wrapper });

    await act(async () => {
      await result.current.mutateAsync("secret-token").catch(() => undefined);
    });

    await waitFor(() => {
      expect(warn).toHaveBeenCalled();
    });
    const logged = JSON.stringify(warn.mock.calls);
    expect(logged).toContain("invalid_token");
    expect(logged).not.toContain("secret-token");
  });
});
