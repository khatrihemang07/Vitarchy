import { screen } from "@testing-library/react";
import { jsonResponse, renderWithClient, stubApi } from "../test-utils";
import { ActivityTrend } from "./ActivityTrend";

const period = { from: "2026-05-06", to: "2026-05-12", tz: "UTC" };
const PATH = "GET /api/time/analysis/activity/trend";

afterEach(() => vi.unstubAllGlobals());

describe("ActivityTrend", () => {
  it("names each app in the legend and gives the weeks in a table, flagging partial weeks", async () => {
    const fetchMock = stubApi({
      [PATH]: () =>
        jsonResponse(200, {
          weeks: [
            { start: "2026-05-04", days: 5 },
            { start: "2026-05-11", days: 2 },
          ],
          series: [{ app: "Code", seconds: [7200, 3600] }],
          other: [0, 0],
        }),
    });
    renderWithClient(<ActivityTrend period={period} />);
    expect((await screen.findAllByText("Code")).length).toBeGreaterThan(0);
    expect(screen.getByRole("cell", { name: "5", hidden: true })).toBeInTheDocument();
    const asked = fetchMock.mock.calls.map((c) => new URL((c)[0].url)).find((u) => u.pathname.endsWith("/trend"));
    expect(asked?.searchParams.get("week_start")).toBe("monday");
  });

  it("says so when there is no Activity", async () => {
    stubApi({ [PATH]: () => jsonResponse(200, { weeks: [{ start: "2026-05-04", days: 7 }], series: [], other: [0] }) });
    renderWithClient(<ActivityTrend period={period} />);
    expect(await screen.findByText("No Activity in this Period.")).toBeInTheDocument();
  });

  it("explains a failure", async () => {
    stubApi({ [PATH]: () => jsonResponse(500, { detail: "boom" }) });
    renderWithClient(<ActivityTrend period={period} />);
    expect(await screen.findByRole("alert")).toBeInTheDocument();
  });
});
