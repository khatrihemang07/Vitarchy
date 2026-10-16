import { screen } from "@testing-library/react";
import { jsonResponse, renderWithClient, stubApi } from "../test-utils";
import { ActivityDayBounds } from "./ActivityDayBounds";

const period = { from: "2026-05-04", to: "2026-05-06", tz: "UTC" };
const PATH = "GET /api/time/analysis/activity/bounds";

afterEach(() => vi.unstubAllGlobals());

describe("ActivityDayBounds", () => {
  it("lists each Day's first and last activity in a table, in the clock's style", async () => {
    stubApi({ [PATH]: () => jsonResponse(200, { days: [{ date: "2026-05-05", first_minute: 570, last_minute: 1500 - 60 }] }) });
    renderWithClient(<ActivityDayBounds period={period} />);
    expect(await screen.findByText("May 5", { selector: "td" })).toBeInTheDocument();
    expect(screen.getAllByRole("row", { hidden: true })).toHaveLength(2);
  });

  it("says so when no Day has Activity", async () => {
    stubApi({ [PATH]: () => jsonResponse(200, { days: [] }) });
    renderWithClient(<ActivityDayBounds period={period} />);
    expect(await screen.findByText("No Activity in this Period.")).toBeInTheDocument();
  });

  it("explains a failure", async () => {
    stubApi({ [PATH]: () => jsonResponse(500, { detail: "boom" }) });
    renderWithClient(<ActivityDayBounds period={period} />);
    expect(await screen.findByRole("alert")).toBeInTheDocument();
  });
});
