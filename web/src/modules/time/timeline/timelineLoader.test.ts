import { QueryClient } from "@tanstack/react-query";
import { DEFAULT_CORE, settingsOptions } from "@/api/settings";
import { jsonResponse, stubApi } from "../test-utils";
import { parseTimelineSearch } from "./timelineSearch";
import { timelineQueryForLoader, warmTimeline } from "./timelineLoader";

afterEach(() => vi.unstubAllGlobals());

const NOW = new Date(2026, 9, 14, 12, 0, 0);

describe("timelineQueryForLoader", () => {
  it("asks for the Period in the URL, cut in the given zone", () => {
    const query = timelineQueryForLoader(parseTimelineSearch({ kind: "week" }), "monday", NOW, "Asia/Kolkata");
    expect(query).toEqual({ kind: "week", from: "2026-10-12", to: "2026-10-18", tz: "Asia/Kolkata" });
  });
});

describe("warmTimeline", () => {
  it("fetches the Timeline summary for the URL's Period", async () => {
    const fetchMock = stubApi({ "GET /api/time/timeline": () => jsonResponse(500, {}) });
    const queryClient = new QueryClient();
    queryClient.setQueryData(settingsOptions.queryKey, { core: DEFAULT_CORE, modules: {} });
    await warmTimeline({ kind: "week" })(queryClient).catch(() => undefined);
    expect(fetchMock.mock.calls.map(([request]) => new URL(request.url).pathname)).toEqual(["/api/time/timeline"]);
  });
});
