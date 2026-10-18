import { screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import type { AwayPreview } from "@/api/time";

import { AwayRulesCard } from "./AwayRulesCard";
import { jsonResponse, renderWithClient, stubApi } from "./test-utils";

afterEach(() => vi.unstubAllGlobals());

const H = 3600;
const preview: AwayPreview = {
  current_secs: 3600,
  watching_cap_secs: 7200,
  watching_apps: ["Winamp", "zoom.us"],
  watching_sites: ["video.example"],
  days: 14,
  choices: [
    { away_after_secs: 0, away_secs_per_day: 19 * H, active_secs_per_day: 4 * H },
    { away_after_secs: 3600, away_secs_per_day: 15 * H, active_secs_per_day: 8 * H },
  ],
};
const PATH = "GET /api/time/timeline/away-preview";

describe("AwayRulesCard", () => {
  it("shows a loading state first", () => {
    stubApi({ [PATH]: () => new Promise<Response>(() => undefined) });
    renderWithClient(<AwayRulesCard />);
    expect(screen.getByText("Loading Away rules…")).toBeInTheDocument();
  });

  it("states the rules in plain words and marks the current row", async () => {
    stubApi({ [PATH]: () => jsonResponse(200, preview) });
    renderWithClient(<AwayRulesCard />);
    expect(await screen.findByText(/or more counts as Away, starting from your last input/)).toBeInTheDocument();
    expect(screen.getByText(/the first/)).toHaveTextContent("2h without input stays Active");
    expect(screen.getByText(/the first/)).toHaveTextContent("Winamp, zoom.us, video.example");
    const current = screen.getByRole("row", { current: true });
    expect(within(current).getByText("1 hour")).toBeInTheDocument();
    expect(within(current).getByText("15h")).toBeInTheDocument();
  });

  it("shows the change of each other threshold and the trade-off", async () => {
    stubApi({ [PATH]: () => jsonResponse(200, preview) });
    renderWithClient(<AwayRulesCard />);
    expect(await screen.findByText("+4h")).toBeInTheDocument();
    expect(screen.getByText("−4h")).toBeInTheDocument();
    expect(screen.getByText(/Higher: more pauses count as Active/)).toBeInTheDocument();
    expect(screen.getByText(/Lower: stricter/)).toBeInTheDocument();
  });

  it("names the window the table looks back over from the data", async () => {
    stubApi({ [PATH]: () => jsonResponse(200, preview) });
    renderWithClient(<AwayRulesCard />);
    expect(await screen.findByText(/over the last 14 Days of Clockify Activity/)).toBeInTheDocument();
  });

  it("invites an Import when the window holds no Clockify Activity", async () => {
    const empty = { ...preview, choices: preview.choices.map((c) => ({ ...c, away_secs_per_day: 0, active_secs_per_day: 0 })) };
    stubApi({ [PATH]: () => jsonResponse(200, empty) });
    renderWithClient(<AwayRulesCard />);
    expect(await screen.findByText(/No Clockify Activity in the last 14 Days/)).toBeInTheDocument();
    expect(screen.queryByRole("table")).not.toBeInTheDocument();
  });

  it("explains a failure and tries again", async () => {
    const handler = vi.fn().mockReturnValueOnce(jsonResponse(500, { detail: "boom" })).mockReturnValue(jsonResponse(200, preview));
    stubApi({ [PATH]: handler });
    renderWithClient(<AwayRulesCard />);
    expect(await screen.findByRole("button", { name: "Try again" })).toBeInTheDocument();
    expect(screen.queryByRole("table")).not.toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Try again" }));
    expect(await screen.findByRole("table")).toBeInTheDocument();
  });
});
