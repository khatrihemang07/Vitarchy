import { screen, within } from "@testing-library/react";
import { jsonResponse, renderWithClient, stubApi } from "../test-utils";
import { ActivityHeatmap } from "./ActivityHeatmap";

const period = { from: "2026-05-04", to: "2026-05-10", tz: "UTC" };
const PATH = "GET /api/time/analysis/activity/heatmap";
const grid = (cells: Record<string, number>) =>
  Array.from({ length: 7 }, (_, day) => Array.from({ length: 24 }, (_, hour) => cells[`${String(day)}-${String(hour)}`] ?? 0));

afterEach(() => vi.unstubAllGlobals());

describe("ActivityHeatmap", () => {
  it("names the busiest weekday and hour and gives every cell in a table", async () => {
    stubApi({ [PATH]: () => jsonResponse(200, { seconds: grid({ "2-21": 3600, "0-9": 600 }) }) });
    renderWithClient(<ActivityHeatmap period={period} />);
    expect(await screen.findByText(/Busiest: Wednesdays/)).toHaveTextContent("1 h in all");
    const table = screen.getByRole("table", { hidden: true });
    expect(within(table).getAllByRole("row", { hidden: true })).toHaveLength(25);
    expect(within(table).getByText("60", { exact: true })).toBeInTheDocument();
  });

  it("says so when no hour has Activity", async () => {
    stubApi({ [PATH]: () => jsonResponse(200, { seconds: grid({}) }) });
    renderWithClient(<ActivityHeatmap period={period} />);
    expect(await screen.findByText("No Activity in this Period.")).toBeInTheDocument();
  });

  it("explains a failure", async () => {
    stubApi({ [PATH]: () => jsonResponse(500, { detail: "boom" }) });
    renderWithClient(<ActivityHeatmap period={period} />);
    expect(await screen.findByRole("alert")).toBeInTheDocument();
  });
});
