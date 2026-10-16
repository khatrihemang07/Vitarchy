import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { jsonResponse, renderWithClient, stubApi } from "../test-utils";
import { ActivityDayTimeline } from "./ActivityDayTimeline";
import { OVERVIEW, overviewBody } from "./overviewFixture";

const period = { from: "2026-05-04", to: "2026-05-06", tz: "UTC" };
const BOUNDS = OVERVIEW;
const LANE = "GET /api/time/analysis/activity/timeline";

const lane = (date: string, spans: object[]) => ({
  date,
  day_start: `${date}T00:00:00Z`,
  day_end: `${date.slice(0, 8)}${String(Number(date.slice(8)) + 1).padStart(2, "0")}T00:00:00Z`,
  spans,
});
const span = (date: string) => ({ start: `${date}T09:00:00Z`, end: `${date}T10:00:00Z`, app: "Code", context: "vitarchy" });

afterEach(() => vi.unstubAllGlobals());

describe("ActivityDayTimeline", () => {
  it("opens on the last Day with Activity and shows its spans in a table", async () => {
    const fetchMock = stubApi({
      [BOUNDS]: () => jsonResponse(200, overviewBody({ bounds: [{ date: "2026-05-04", first_minute: 0, last_minute: 60 }, { date: "2026-05-05", first_minute: 0, last_minute: 60 }] })),
      [LANE]: (request) => jsonResponse(200, lane(new URL(request.url).searchParams.get("day") ?? "", [span("2026-05-05")])),
    });
    renderWithClient(<ActivityDayTimeline period={period} />);
    expect(await screen.findByLabelText("Day")).toHaveValue("2026-05-05");
    expect((await screen.findAllByText("vitarchy")).length).toBeGreaterThan(0);
    const laneCall = fetchMock.mock.calls.map((c) => new URL((c)[0].url)).find((u) => u.pathname.endsWith("/timeline"));
    expect(laneCall?.searchParams.get("day")).toBe("2026-05-05");
  });

  it("moves to the previous Day, and an empty Day says so while keeping the picker", async () => {
    stubApi({
      [BOUNDS]: () => jsonResponse(200, overviewBody({ bounds: [{ date: "2026-05-05", first_minute: 0, last_minute: 60 }] })),
      [LANE]: (request) => {
        const day = new URL(request.url).searchParams.get("day") ?? "";
        return jsonResponse(200, lane(day, day === "2026-05-05" ? [span(day)] : []));
      },
    });
    renderWithClient(<ActivityDayTimeline period={period} />);
    await userEvent.click(await screen.findByRole("button", { name: "Previous day" }));
    expect(await screen.findByText("No Activity on this day.")).toBeInTheDocument();
    expect(screen.getByLabelText("Day")).toHaveValue("2026-05-04");
    expect(screen.getByRole("button", { name: "Previous day" })).toBeDisabled();
  });

  it("says so when the Period has no Activity at all", async () => {
    stubApi({ [BOUNDS]: () => jsonResponse(200, overviewBody({ bounds: [] })) });
    renderWithClient(<ActivityDayTimeline period={period} />);
    expect(await screen.findByText("No Activity in this Period.")).toBeInTheDocument();
    expect(screen.queryByLabelText("Day")).not.toBeInTheDocument();
  });

  it("explains a failure", async () => {
    stubApi({ [BOUNDS]: () => jsonResponse(500, { detail: "boom" }) });
    renderWithClient(<ActivityDayTimeline period={period} />);
    expect(await screen.findByRole("alert")).toBeInTheDocument();
  });
});
