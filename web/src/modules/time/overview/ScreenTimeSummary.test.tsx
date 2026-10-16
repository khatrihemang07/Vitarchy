import { screen, within } from "@testing-library/react";
import { OVERVIEW, overviewBody } from "../activity/overviewFixture";
import { jsonResponse, renderWithClient, stubApi } from "../test-utils";
import { ScreenTimeSummary } from "./ScreenTimeSummary";

const period = { from: "2026-05-04", to: "2026-05-10", tz: "UTC" };
const previous = { from: "2026-04-27", to: "2026-05-03", tz: "UTC" };

const totals = (seconds: number) => ({
  total_seconds: seconds,
  apps: seconds === 0 ? [] : [
    { app: "Code", seconds: seconds * 0.6, contexts: [{ name: "vitarchy", seconds: seconds * 0.6, titles: [] }] },
    { app: "Slack", seconds: seconds * 0.4, contexts: [{ name: "Slack", seconds: seconds * 0.4, titles: [] }] },
  ],
});

/** Answers by the `from` date, so the current and previous Period differ. */
const byPeriod = (current: number, before: number) => (request: Request) =>
  jsonResponse(200, overviewBody({ totals: totals(new URL(request.url).searchParams.get("from") === period.from ? current : before) }));

afterEach(() => vi.unstubAllGlobals());

describe("ScreenTimeSummary", () => {
  it("shows the total, the change against the previous Period, and the top apps and Contexts", async () => {
    stubApi({ [OVERVIEW]: byPeriod(3 * 3600, 2 * 3600) });
    renderWithClient(<ScreenTimeSummary period={period} previous={previous} />);
    expect(await screen.findByText("3 h")).toBeInTheDocument();
    expect(await screen.findByText("1 h more than the previous Period (+50%)")).toBeInTheDocument();
    const apps = screen.getByRole("list", { name: "Top apps" });
    expect(within(apps).getByText("Code")).toBeInTheDocument();
    expect(within(apps).getByText("Slack")).toBeInTheDocument();
    const contexts = screen.getByRole("list", { name: "Top Contexts" });
    expect(within(contexts).getByText("vitarchy")).toBeInTheDocument();
    expect(within(contexts).queryByText("Slack")).not.toBeInTheDocument();
  });

  it("leaves out the comparison when the previous Period had no Activity", async () => {
    stubApi({ [OVERVIEW]: byPeriod(3600, 0) });
    renderWithClient(<ScreenTimeSummary period={period} previous={previous} />);
    await screen.findByText("1 h");
    expect(screen.queryByText(/previous Period/)).not.toBeInTheDocument();
  });

  it("says so when there is no Activity, and explains a failure", async () => {
    stubApi({ [OVERVIEW]: byPeriod(0, 0) });
    const { unmount } = renderWithClient(<ScreenTimeSummary period={period} previous={previous} />);
    expect(await screen.findByText("No Activity in this Period.")).toBeInTheDocument();
    unmount();
    stubApi({ [OVERVIEW]: () => jsonResponse(500, { detail: "boom" }) });
    renderWithClient(<ScreenTimeSummary period={period} previous={previous} />);
    expect(await screen.findByRole("alert")).toBeInTheDocument();
  });
});
