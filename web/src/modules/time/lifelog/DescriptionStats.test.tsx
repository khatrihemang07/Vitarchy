import { screen } from "@testing-library/react";
import { jsonResponse, renderWithClient, stubApi } from "../test-utils";
import { DescriptionStats } from "./DescriptionStats";
import { LIFELOG_OVERVIEW, LOGGED, lifeLogBody } from "./overviewFixture";

const period = { from: "2026-05-04", to: "2026-05-10", tz: "UTC" };

afterEach(() => vi.unstubAllGlobals());

describe("DescriptionStats", () => {
  it("writes each habit with its usual time, range and number of days", async () => {
    stubApi({ [LIFELOG_OVERVIEW]: () => jsonResponse(200, LOGGED) });
    renderWithClient(<DescriptionStats period={period} />);
    expect(await screen.findByRole("heading", { name: "sleep" })).toBeInTheDocument();
    expect(screen.getByText("6 days")).toBeInTheDocument();
    expect(screen.getByText(/(23|11):40.*(00|12):25/)).toBeInTheDocument();
    expect(screen.getByText("7 h 30 min")).toBeInTheDocument();
  });

  it("says so when no description comes back often enough, and explains a failure", async () => {
    stubApi({ [LIFELOG_OVERVIEW]: () => jsonResponse(200, lifeLogBody()) });
    const { unmount } = renderWithClient(<DescriptionStats period={period} />);
    expect(await screen.findByText(/enough days/)).toBeInTheDocument();
    unmount();
    stubApi({ [LIFELOG_OVERVIEW]: () => jsonResponse(500, { detail: "boom" }) });
    renderWithClient(<DescriptionStats period={period} />);
    expect(await screen.findByRole("alert")).toBeInTheDocument();
  });
});
