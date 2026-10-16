import { screen, within } from "@testing-library/react";
import { LIFELOG_OVERVIEW, LOGGED, lifeLogBody } from "../lifelog/overviewFixture";
import { OCCURRENCES, occurrencesBody } from "../lifelog/routines/fixtures";
import { jsonResponse, renderWithClient, stubApi } from "../test-utils";
import { LifeLogSummary } from "./LifeLogSummary";

const period = { from: "2026-05-04", to: "2026-05-05", tz: "UTC" };

afterEach(() => vi.unstubAllGlobals());

describe("LifeLogSummary", () => {
  it("shows hours per project, the Unlogged time and each Routine's door-to-door time", async () => {
    stubApi({
      [LIFELOG_OVERVIEW]: () => jsonResponse(200, LOGGED),
      [OCCURRENCES]: () => jsonResponse(200, { routines: [occurrencesBody] }),
    });
    renderWithClient(<LifeLogSummary period={period} />);
    const projects = await screen.findByRole("list", { name: "Hours per project" });
    expect(within(projects).getByText("sleep")).toBeInTheDocument();
    expect(within(projects).getByText("office")).toBeInTheDocument();
    // 50400 + 79200 s unlogged of 172800 s elapsed.
    expect(screen.getByText(/Unlogged, 75% of the time that has passed/)).toBeInTheDocument();
    const routines = await screen.findByRole("list", { name: "Routines" });
    expect(within(routines).getByText("Office day")).toBeInTheDocument();
    expect(within(routines).getByText("9 h 15 min")).toBeInTheDocument();
    expect(within(routines).getByText(/1 complete day$/)).toBeInTheDocument();
  });

  it("points to the Life log when no Routine is saved", async () => {
    stubApi({
      [LIFELOG_OVERVIEW]: () => jsonResponse(200, LOGGED),
      [OCCURRENCES]: () => jsonResponse(200, { routines: [] }),
    });
    renderWithClient(<LifeLogSummary period={period} />);
    expect(await screen.findByText(/Save a Routine in the Life log tab/)).toBeInTheDocument();
  });

  it("says so when nothing is logged, and explains a failure", async () => {
    stubApi({ [LIFELOG_OVERVIEW]: () => jsonResponse(200, lifeLogBody()) });
    const { unmount } = renderWithClient(<LifeLogSummary period={period} />);
    expect(await screen.findByText("No Time Entries in this Period.")).toBeInTheDocument();
    unmount();
    stubApi({ [LIFELOG_OVERVIEW]: () => jsonResponse(500, { detail: "boom" }) });
    renderWithClient(<LifeLogSummary period={period} />);
    expect(await screen.findByRole("alert")).toBeInTheDocument();
  });
});
