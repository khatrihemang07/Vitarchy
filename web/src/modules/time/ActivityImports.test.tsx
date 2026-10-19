import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import type { TimeStatus } from "@/api/time";

import { ActivityImports } from "./ActivityImports";
import { disconnected, jsonResponse, noActivity, renderWithClient, stubApi } from "./test-utils";

afterEach(() => vi.unstubAllGlobals());

const snapshot = new Date(Date.now() - 3_600_000).toISOString();
const ready: TimeStatus = {
  ...disconnected,
  activity: [
    { ...noActivity("toggl"), snapshot_taken_at: snapshot },
    { ...noActivity("clockify"), snapshot_taken_at: snapshot },
  ],
};

describe("ActivityImports", () => {
  it("shows a loading state while the status is pending", () => {
    stubApi({ "GET /api/time/status": () => new Promise<Response>(() => undefined) });
    renderWithClient(<ActivityImports />);
    expect(screen.getByText("Loading status…")).toBeInTheDocument();
  });

  it("shows an error when the status cannot be loaded", async () => {
    stubApi({ "GET /api/time/status": () => new Response("", { status: 502 }) });
    renderWithClient(<ActivityImports />);
    expect(await screen.findByRole("alert")).toBeInTheDocument();
  });

  it("shows one card per Source, in the status order", async () => {
    stubApi({ "GET /api/time/status": () => jsonResponse(200, disconnected) });
    renderWithClient(<ActivityImports />);
    const titles = await screen.findAllByText(/^Import (Toggl|Clockify) activity$/, {
      selector: "[data-slot=card-title]",
    });
    expect(titles.map((title) => title.textContent)).toEqual(["Import Toggl activity", "Import Clockify activity"]);
  });

  it("refreshes the status after an Import", async () => {
    const after: TimeStatus = {
      ...ready,
      activity: [
        {
          ...noActivity("toggl"),
          snapshot_taken_at: snapshot,
          activity_count: 10,
          app_count: 3,
          oldest: "2026-04-01T10:00:00Z",
          newest: "2026-04-05T10:00:00Z",
        },
        { ...noActivity("clockify"), snapshot_taken_at: snapshot },
      ],
    };
    stubApi({
      "GET /api/time/status": [() => jsonResponse(200, ready), () => jsonResponse(200, after)],
      "POST /api/time/activity/toggl/import": () => jsonResponse(200, { read: 10, inserted: 10 }),
    });
    renderWithClient(<ActivityImports />);

    await userEvent.click(await screen.findByRole("button", { name: "Import Toggl activity" }));

    expect(await screen.findByText(/10 Activity rows from 3 apps/)).toBeInTheDocument();
  });
});
