import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { jsonResponse, renderWithClient, stubApi } from "../test-utils";
import { InsightList } from "./InsightList";

const INSIGHTS = "GET /api/time/analysis/insights";
const period = { from: "2026-05-01", to: "2026-05-31", tz: "UTC" };

const insight = (name: string, extra: Record<string, unknown> = {}) => ({
  kind: "typical_duration",
  subject: { kind: "routine_step", name },
  measure: "length",
  numbers: { value: 30, p25: 28, p75: 34, samples: 10 },
  score: 40,
  days: [],
  ...extra,
});

afterEach(() => vi.unstubAllGlobals());

describe("InsightList", () => {
  it("leads with the top five and shows the rest on request", async () => {
    const many = Array.from({ length: 7 }, (_, i) => insight(`Step ${String(i)}`));
    stubApi({ [INSIGHTS]: () => jsonResponse(200, { insights: many }) });
    renderWithClient(<InsightList period={period} onOpen={() => undefined} />);
    expect(await screen.findByText(/Step 0 usually takes 30 min/)).toBeInTheDocument();
    expect(screen.getAllByRole("listitem")).toHaveLength(5);
    await userEvent.click(screen.getByRole("button", { name: "Show all 7" }));
    expect(screen.getAllByRole("listitem")).toHaveLength(7);
    await userEvent.click(screen.getByRole("button", { name: "Show fewer" }));
    expect(screen.getAllByRole("listitem")).toHaveLength(5);
  });

  it("offers no Show all when there are five or fewer", async () => {
    stubApi({ [INSIGHTS]: () => jsonResponse(200, { insights: [insight("Office")] }) });
    renderWithClient(<InsightList period={period} onOpen={() => undefined} />);
    await screen.findByText(/Office usually takes/);
    expect(screen.queryByRole("button", { name: /Show/ })).not.toBeInTheDocument();
  });

  it("opens the tab and Day an Insight is about", async () => {
    const onOpen = vi.fn();
    const outlier = insight("Office", { kind: "outlier", numbers: { value: 612, delta: 132, reference: 480, samples: 9 }, days: ["2026-05-12"] });
    stubApi({ [INSIGHTS]: () => jsonResponse(200, { insights: [outlier] }) });
    renderWithClient(<InsightList period={period} onOpen={onOpen} />);
    await userEvent.click(await screen.findByRole("button", { name: /Longest Office day/ }));
    expect(onOpen).toHaveBeenCalledWith({ tab: "life-log", day: "2026-05-12" });
  });

  it("says so when nothing stands out, and explains a failure", async () => {
    stubApi({ [INSIGHTS]: () => jsonResponse(200, { insights: [] }) });
    const { unmount } = renderWithClient(<InsightList period={period} onOpen={() => undefined} />);
    expect(await screen.findByText(/Nothing stands out yet/)).toBeInTheDocument();
    unmount();
    stubApi({ [INSIGHTS]: () => jsonResponse(500, { detail: "boom" }) });
    renderWithClient(<InsightList period={period} onOpen={() => undefined} />);
    expect(await screen.findByRole("alert")).toBeInTheDocument();
  });
});
