import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { jsonResponse, renderWithClient, stubApi } from "../test-utils";
import { LifeLogDayTimeline } from "./LifeLogDayTimeline";
import { LIFELOG_OVERVIEW, LIFELOG_TIMELINE, LOGGED, lifeLogBody } from "./overviewFixture";

const period = { from: "2026-05-04", to: "2026-05-05", tz: "UTC" };
const lane = (date: string, spans: object[]) => ({ date, day_start: `${date}T00:00:00Z`, day_end: `${date.slice(0, 8)}${String(Number(date.slice(8)) + 1).padStart(2, "0")}T00:00:00Z`, spans });
const span = (date: string) => ({ start: `${date}T09:00:00Z`, end: `${date}T17:00:00Z`, project: "office", color: "#2a78d6", description: "inside_office" });

afterEach(() => vi.unstubAllGlobals());

describe("LifeLogDayTimeline", () => {
  it("opens on the last Day with entries, labelling each by description and colouring it by project", async () => {
    const fetchMock = stubApi({
      [LIFELOG_OVERVIEW]: () => jsonResponse(200, LOGGED),
      [LIFELOG_TIMELINE]: (request) => jsonResponse(200, lane(new URL(request.url).searchParams.get("day") ?? "", [span("2026-05-05")])),
    });
    renderWithClient(<LifeLogDayTimeline period={period} />);
    expect(await screen.findByLabelText("Day")).toHaveValue("2026-05-05");
    expect((await screen.findAllByText("office")).length).toBeGreaterThan(0);
    expect(screen.getByRole("cell", { name: "inside_office (office)", hidden: true })).toBeInTheDocument();
    const asked = fetchMock.mock.calls.map((c) => new URL(c[0].url)).find((u) => u.pathname.endsWith("/timeline"));
    expect(asked?.searchParams.get("day")).toBe("2026-05-05");
  });

  it("moves to the previous Day, and an empty Day says so while keeping the picker", async () => {
    stubApi({
      [LIFELOG_OVERVIEW]: () => jsonResponse(200, LOGGED),
      [LIFELOG_TIMELINE]: (request) => {
        const day = new URL(request.url).searchParams.get("day") ?? "";
        return jsonResponse(200, lane(day, day === "2026-05-05" ? [span(day)] : []));
      },
    });
    renderWithClient(<LifeLogDayTimeline period={period} />);
    await userEvent.click(await screen.findByRole("button", { name: "Previous day" }));
    expect(await screen.findByText("No Time Entries on this day.")).toBeInTheDocument();
    expect(screen.getByLabelText("Day")).toHaveValue("2026-05-04");
  });

  it("says so when the Period has no entries, and explains a failure", async () => {
    stubApi({ [LIFELOG_OVERVIEW]: () => jsonResponse(200, lifeLogBody()) });
    const { unmount } = renderWithClient(<LifeLogDayTimeline period={period} />);
    expect(await screen.findByText("No Time Entries in this Period.")).toBeInTheDocument();
    unmount();
    stubApi({ [LIFELOG_OVERVIEW]: () => jsonResponse(500, { detail: "boom" }) });
    renderWithClient(<LifeLogDayTimeline period={period} />);
    expect(await screen.findByRole("alert")).toBeInTheDocument();
  });
});
