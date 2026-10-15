import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import type { ReactNode } from "react";
import { Wallet } from "lucide-react";
import { DEFAULT_CORE } from "./core-sections";
import { useModuleSettings } from "../module-manifest";

const declaration = {
  label: "Finance",
  icon: Wallet,
  fields: [
    {
      kind: "select",
      key: "currency",
      label: "Currency",
      default: "GBP",
      options: [
        { value: "GBP", label: "GBP" },
        { value: "EUR", label: "EUR" },
      ],
    },
    { kind: "toggle", key: "roundUp", label: "Round up", default: false },
  ],
} as const;

function serve(stored: Record<string, unknown>) {
  let modules: Record<string, unknown> = { finance: stored };
  const reply = (body: unknown) =>
    new Response(JSON.stringify(body), { headers: { "content-type": "application/json" } });
  vi.stubGlobal("fetch", async (request: Request) => {
    if (request.method === "GET") return reply({ core: DEFAULT_CORE, modules });
    const body: unknown = await request.json();
    modules = { ...modules, finance: body };
    return reply(body);
  });
  return () => modules.finance;
}

const wrapper = ({ children }: { children: ReactNode }) => (
  <QueryClientProvider client={new QueryClient()}>{children}</QueryClientProvider>
);

afterEach(() => vi.unstubAllGlobals());

describe("useModuleSettings", () => {
  it("gives the declared defaults until the stored values load, then the stored ones", async () => {
    serve({ currency: "EUR", roundUp: "not a boolean", unknown: 1 });
    const { result } = renderHook(() => useModuleSettings("finance", declaration), { wrapper });

    expect(result.current.values).toEqual({ currency: "GBP", roundUp: false });
    await waitFor(() => {
      expect(result.current.values.currency).toBe("EUR");
    });
    expect(result.current.values).toEqual({ currency: "EUR", roundUp: false });
  });

  it("saves one key under the Module id, keeping the others", async () => {
    const stored = serve({ currency: "EUR" });
    const { result } = renderHook(() => useModuleSettings("finance", declaration), { wrapper });
    await waitFor(() => {
      expect(result.current.values.currency).toBe("EUR");
    });

    act(() => {
      result.current.save("roundUp", true);
    });

    await waitFor(() => {
      expect(stored()).toEqual({ currency: "EUR", roundUp: true });
    });
    await waitFor(() => {
      expect(result.current.values.roundUp).toBe(true);
    });
  });

  it("types values from a const declaration", () => {
    serve({});
    const { result } = renderHook(() => useModuleSettings("finance", declaration), { wrapper });
    const currency: "GBP" | "EUR" = result.current.values.currency;
    const roundUp: boolean = result.current.values.roundUp;
    expect([currency, roundUp]).toEqual(["GBP", false]);
  });
});
