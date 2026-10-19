import { QueryClient } from "@tanstack/react-query";
import { DEFAULT_CORE, settingsOptions } from "@/api/settings";
import { facetsOptions, type Facets } from "@/api/time";
import { jsonResponse, stubApi } from "../test-utils";
import { reportQueryForLoader, warmReport } from "./reportLoader";
import { parseReportSearch } from "./reportSearch";

afterEach(() => {
  vi.unstubAllGlobals();
  localStorage.clear();
});

const facets: Facets = { projects: [{ id: 5, name: "Site", color: null, client_id: null, client_name: null }], tags: [], clients: [] } as unknown as Facets;
const NOW = new Date(2026, 9, 14, 12, 0, 0);

describe("reportQueryForLoader", () => {
  it("asks for the Period and filters the URL names", () => {
    const search = parseReportSearch({ kind: "week", project: [5] });
    const query = reportQueryForLoader({ search, facets, weekStart: "monday", now: NOW, tz: "Asia/Kolkata" });
    expect(query).toMatchObject({ kind: "week", from: "2026-10-12", to: "2026-10-18", tz: "Asia/Kolkata", project: [5] });
  });

  it("follows the week start", () => {
    const search = parseReportSearch({ kind: "week" });
    const sunday = reportQueryForLoader({ search, facets, weekStart: "sunday", now: NOW, tz: "UTC" });
    expect(sunday.from).toBe("2026-10-11");
  });
});

describe("warmReport", () => {
  it("fetches the Report after the settings and filter choices, using the cached ones", async () => {
    const fetchMock = stubApi({ "GET /api/time/report": () => jsonResponse(500, {}) });
    const queryClient = new QueryClient();
    queryClient.setQueryData(settingsOptions.queryKey, { core: DEFAULT_CORE, modules: {} });
    queryClient.setQueryData(facetsOptions.queryKey, facets);
    await warmReport({ kind: "week" })(queryClient).catch(() => undefined);
    expect(fetchMock.mock.calls.map(([request]) => new URL(request.url).pathname)).toEqual(["/api/time/report"]);
  });
});
