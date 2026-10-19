import { QueryClient, queryOptions } from "@tanstack/react-query";
import { RequestError } from "@/api/callApi";
import { warm, warmCache, warmCacheWith } from "./prefetch";

const page = (key: string, fetch: () => Promise<string>) => queryOptions({ queryKey: [key], queryFn: fetch, retry: false });

describe("warmCache", () => {
  it("fills the router's query cache with the page's queries", async () => {
    const queryClient = new QueryClient();
    await warmCache([warm(page("a", () => Promise.resolve("one")))])({ context: { queryClient } });
    expect(queryClient.getQueryData(["a"])).toBe("one");
  });

  it("does not fetch again what is already cached, however old", async () => {
    const queryClient = new QueryClient();
    queryClient.setQueryData(["a"], "cached");
    const fetch = vi.fn(() => Promise.resolve("fresh"));
    await warmCache([warm(page("a", fetch))])({ context: { queryClient } });
    expect(fetch).not.toHaveBeenCalled();
  });

  it("does nothing without a query client in the context", async () => {
    const fetch = vi.fn(() => Promise.resolve("x"));
    await warmCache([warm(page("a", fetch))])({ context: {} });
    expect(fetch).not.toHaveBeenCalled();
  });

  it("does not reject when a query fails, so the page can show the failure", async () => {
    const queryClient = new QueryClient();
    const failing = page("a", () => Promise.reject(new Error("down")));
    await expect(warmCache([warm(failing)])({ context: { queryClient } })).resolves.toBeUndefined();
  });

  it("logs a failed warm with its status but never what the error said", async () => {
    const warn = vi.spyOn(console, "warn").mockImplementation(() => undefined);
    const queryClient = new QueryClient();
    const failing = page("a", () => Promise.reject(new RequestError("secret detail", "rejected", 500)));
    await warmCache([warm(failing)])({ context: { queryClient } });
    const logged = warn.mock.calls[0] ?? [];
    expect(String(logged[0])).toContain("warm failed");
    expect(logged).toContainEqual(expect.objectContaining({ status: 500 }));
    expect(JSON.stringify(logged)).not.toContain("secret detail");
    warn.mockRestore();
  });

  it("builds the queries from the loader's deps", async () => {
    const queryClient = new QueryClient();
    await warmCacheWith((deps: { id: string }) => [warm(page(deps.id, () => Promise.resolve("by deps")))])({ context: { queryClient }, deps: { id: "b" } });
    expect(queryClient.getQueryData(["b"])).toBe("by deps");
  });
});
