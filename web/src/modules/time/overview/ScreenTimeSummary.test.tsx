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

/** `count` finished Days of `seconds` each. */
const days = (count: number, seconds: number) => Array.from({ length: count }, (_, i) => ({ date: `2026-05-${String(i + 1).padStart(2, "0")}`, seconds }));

interface Figures { seconds: number; days: number }

/** Answers by the `from` date, so the current and previous Period differ. */
const byPeriod = (current: Figures, before: Figures) => (request: Request) => {
  const figures = new URL(request.url).searchParams.get("from") === period.from ? current : before;
  return jsonResponse(
    200,
    overviewBody({
      totals: totals(figures.seconds),
      top_contexts: figures.seconds === 0 ? [] : [{ name: "vitarchy", seconds: figures.seconds * 0.6 }],
      day_totals: days(figures.days, figures.days === 0 ? 0 : figures.seconds / figures.days),
    }),
  );
};

afterEach(() => vi.unstubAllGlobals());

describe("ScreenTimeSummary", () => {
  it("shows the total, the change against the previous Period, and the top apps and Contexts", async () => {
    stubApi({ [OVERVIEW]: byPeriod({ seconds: 3 * 3600, days: 2 }, { seconds: 14 * 3600, days: 7 }) });
    renderWithClient(<ScreenTimeSummary period={period} previous={previous} />);
    expect(await screen.findByText("3 h")).toBeInTheDocument();
    expect(await screen.findByText("30 min less per day than the previous Period (-25%)")).toBeInTheDocument();
    const apps = screen.getByRole("list", { name: "Top apps" });
    expect(within(apps).getByText("Code")).toBeInTheDocument();
    expect(within(apps).getByText("Slack")).toBeInTheDocument();
    const contexts = screen.getByRole("list", { name: "Top Contexts" });
    expect(within(contexts).getByText("vitarchy")).toBeInTheDocument();
    expect(within(contexts).queryByText("Slack")).not.toBeInTheDocument();
  });

  it("lists the Contexts the server ranked, without working them out again", async () => {
    stubApi({
      [OVERVIEW]: () => jsonResponse(200, overviewBody({ totals: totals(3600), top_contexts: [{ name: "Ranked by server", seconds: 100 }], day_totals: days(1, 3600) })),
    });
    renderWithClient(<ScreenTimeSummary period={period} previous={previous} />);
    const contexts = await screen.findByRole("list", { name: "Top Contexts" });
    expect(within(contexts).getByText("Ranked by server")).toBeInTheDocument();
    expect(within(contexts).queryByText("vitarchy")).not.toBeInTheDocument();
  });

  it("leaves out the comparison when the previous Period had no Activity", async () => {
    stubApi({ [OVERVIEW]: byPeriod({ seconds: 3600, days: 1 }, { seconds: 0, days: 0 }) });
    renderWithClient(<ScreenTimeSummary period={period} previous={previous} />);
    await screen.findByText("1 h");
    expect(screen.queryByText(/previous Period/)).not.toBeInTheDocument();
  });

  it("says so when there is no Activity, and explains a failure", async () => {
    stubApi({ [OVERVIEW]: byPeriod({ seconds: 0, days: 0 }, { seconds: 0, days: 0 }) });
    const { unmount } = renderWithClient(<ScreenTimeSummary period={period} previous={previous} />);
    expect(await screen.findByText("No Activity in this Period.")).toBeInTheDocument();
    unmount();
    stubApi({ [OVERVIEW]: () => jsonResponse(500, { detail: "boom" }) });
    renderWithClient(<ScreenTimeSummary period={period} previous={previous} />);
    expect(await screen.findByRole("alert")).toBeInTheDocument();
  });
});
