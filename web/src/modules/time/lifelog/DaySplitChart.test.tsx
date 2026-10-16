import { screen } from "@testing-library/react";
import { jsonResponse, renderWithClient, stubApi } from "../test-utils";
import { DaySplitChart } from "./DaySplitChart";
import { LIFELOG_OVERVIEW, LOGGED, lifeLogBody } from "./overviewFixture";

const period = { from: "2026-05-04", to: "2026-05-05", tz: "UTC" };

afterEach(() => vi.unstubAllGlobals());

describe("DaySplitChart", () => {
  it("names every project and the unlogged time in the legend and lists each Day in a table", async () => {
    const fetchMock = stubApi({ [LIFELOG_OVERVIEW]: () => jsonResponse(200, LOGGED) });
    renderWithClient(<DaySplitChart period={period} />);
    expect((await screen.findAllByText("office")).length).toBeGreaterThan(0);
    expect(screen.getAllByText("Unlogged").length).toBeGreaterThan(0);
    expect(screen.getByRole("cell", { name: "8 h", hidden: true })).toBeInTheDocument();
    const asked = new URL((fetchMock.mock.calls[0]?.[0] as Request).url);
    expect(asked.searchParams.get("tz")).toBe("UTC");
  });

  it("says so when nothing is logged", async () => {
    stubApi({ [LIFELOG_OVERVIEW]: () => jsonResponse(200, lifeLogBody()) });
    renderWithClient(<DaySplitChart period={period} />);
    expect(await screen.findByText("No Time Entries in this Period.")).toBeInTheDocument();
  });

  it("explains a failure", async () => {
    stubApi({ [LIFELOG_OVERVIEW]: () => jsonResponse(500, { detail: "boom" }) });
    renderWithClient(<DaySplitChart period={period} />);
    expect(await screen.findByRole("alert")).toBeInTheDocument();
  });
});
