import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { jsonResponse, renderWithClient, stubApi } from "../test-utils";
import { ActivityTotals } from "./ActivityTotals";
import { OVERVIEW, overviewBody } from "./overviewFixture";

const period = { from: "2026-05-04", to: "2026-05-10", tz: "UTC" };
const PATH = OVERVIEW;

const totals = {
  total_seconds: 7200 + 1800,
  apps: [
    {
      app: "Code",
      seconds: 7200,
      contexts: [
        { name: "vitarchy", seconds: 5400, titles: [{ title: "main.rs — vitarchy", seconds: 5400 }] },
        { name: "journal", seconds: 1800, titles: [] },
      ],
    },
    { app: "Spotify", seconds: 1800, contexts: [{ name: "Spotify", seconds: 1800, titles: [{ title: "Some Song", seconds: 1800 }] }] },
  ],
};

afterEach(() => vi.unstubAllGlobals());

describe("ActivityTotals", () => {
  it("lists apps longest first with time and share of the total", async () => {
    stubApi({ [PATH]: () => jsonResponse(200, overviewBody({ totals })) });
    renderWithClient(<ActivityTotals period={period} />);
    expect(await screen.findByText("Code")).toBeInTheDocument();
    expect(screen.getByText("2 h")).toBeInTheDocument();
    expect(screen.getByText("80%")).toBeInTheDocument();
    expect(screen.getByText(/2 apps/)).toBeInTheDocument();
  });

  it("opens an app to its Contexts and a Context to its titles", async () => {
    stubApi({ [PATH]: () => jsonResponse(200, overviewBody({ totals })) });
    renderWithClient(<ActivityTotals period={period} />);
    expect(screen.queryByText("vitarchy")).not.toBeInTheDocument();
    await userEvent.click(await screen.findByRole("button", { name: /Code/ }));
    await userEvent.click(screen.getByRole("button", { name: /vitarchy/ }));
    expect(screen.getByText("main.rs — vitarchy")).toBeInTheDocument();
  });

  it("goes straight to titles for an app with one Context", async () => {
    stubApi({ [PATH]: () => jsonResponse(200, overviewBody({ totals })) });
    renderWithClient(<ActivityTotals period={period} />);
    await userEvent.click(await screen.findByRole("button", { name: /Spotify/ }));
    expect(screen.getByText("Some Song")).toBeInTheDocument();
  });

  it("asks for the Period in the browser's zone", async () => {
    const fetchMock = stubApi({ [PATH]: () => jsonResponse(200, overviewBody({ totals })) });
    renderWithClient(<ActivityTotals period={period} />);
    await screen.findByText("Code");
    const asked = fetchMock.mock.calls.map((c) => new URL(c[0].url)).find((u) => u.pathname.endsWith("/overview"));
    expect(Object.fromEntries(asked?.searchParams ?? [])).toEqual({ from: "2026-05-04", to: "2026-05-10", tz: "UTC", week_start: "monday" });
  });

  it("says so when the Period has no Activity", async () => {
    stubApi({ [PATH]: () => jsonResponse(200, overviewBody()) });
    renderWithClient(<ActivityTotals period={period} />);
    expect(await screen.findByText("No Activity in this Period.")).toBeInTheDocument();
  });

  it("explains a failure", async () => {
    stubApi({ [PATH]: () => jsonResponse(400, { detail: "bad zone", code: "unknown_time_zone" }) });
    renderWithClient(<ActivityTotals period={period} />);
    expect(await screen.findByRole("alert")).toBeInTheDocument();
  });

  it("shows a loading note first", () => {
    stubApi({ [PATH]: () => new Promise<Response>(() => undefined) });
    renderWithClient(<ActivityTotals period={period} />);
    expect(screen.getByRole("status")).toBeInTheDocument();
  });
});
