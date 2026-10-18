import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { useAwayPreview } from "./awayPreview";

afterEach(() => {
  vi.unstubAllGlobals();
});

const wrapper = ({ children }: { children: ReactNode }) => (
  <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}>{children}</QueryClientProvider>
);

const json = (status: number, body: unknown) =>
  vi.fn(() => Promise.resolve(new Response(JSON.stringify(body), { status, headers: { "content-type": status >= 400 ? "application/problem+json" : "application/json" } })));

describe("useAwayPreview", () => {
  it("loads the preview from the server", async () => {
    const preview = { current_secs: 3600, watching_cap_secs: 7200, choices: [] };
    const fetchMock = json(200, preview);
    vi.stubGlobal("fetch", fetchMock);
    const { result } = renderHook(() => useAwayPreview(), { wrapper });
    await waitFor(() => { expect(result.current.data).toEqual(preview); });
    expect(new URL((fetchMock.mock.calls[0] as unknown as [Request])[0].url).pathname).toBe("/api/time/timeline/away-preview");
  });

  it("reports a failure as an error, not a throw", async () => {
    vi.stubGlobal("fetch", json(500, { title: "Internal Server Error", status: 500 }));
    const { result } = renderHook(() => useAwayPreview(), { wrapper });
    await waitFor(() => { expect(result.current.isError).toBe(true); });
  });
});
