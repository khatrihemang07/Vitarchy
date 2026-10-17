import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import type { Source, SourceActivityStatus } from "@/api/time";

import { ActivityImportCard } from "./ActivityImportCard";
import { jsonResponse, noActivity, renderWithClient, stubApi } from "./test-utils";

afterEach(() => vi.unstubAllGlobals());

const hoursAgo = (hours: number) => new Date(Date.now() - hours * 3_600_000).toISOString();

function withSnapshot(source: Source): SourceActivityStatus {
  return { ...noActivity(source), snapshot_taken_at: hoursAgo(3) };
}

function imported(source: Source): SourceActivityStatus {
  return {
    ...withSnapshot(source),
    activity_count: 10,
    app_count: 3,
    oldest: "2026-04-01T10:00:00Z",
    newest: "2026-04-05T10:00:00Z",
    last_import: {
      kind: "import",
      started_at: "2026-04-10T12:00:00Z",
      finished_at: "2026-04-10T12:00:02Z",
      outcome: "success",
      counts: { inserted: 10, skipped: 1 } as never,
      error_code: null,
    },
  };
}

describe("ActivityImportCard", () => {
  it.each([
    ["toggl", "Import Toggl activity", /Toggl Track app/],
    ["clockify", "Import Clockify activity", /Clockify Desktop app/],
  ] as const)("describes %s in its own words", (source, title, description) => {
    renderWithClient(<ActivityImportCard activity={noActivity(source)} />);
    expect(screen.getByText(title, { selector: "[data-slot=card-title]" })).toBeInTheDocument();
    expect(screen.getByText(description)).toBeInTheDocument();
  });

  it("tells the person to install the helper when there is no Snapshot, and disables Import", () => {
    renderWithClient(<ActivityImportCard activity={noActivity("toggl")} />);
    expect(screen.getByText(/No Snapshot yet/)).toBeInTheDocument();
    expect(screen.getByText("make helper-install")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Import Toggl activity" })).toBeDisabled();
  });

  it("shows the Snapshot's age and invites a first Import", () => {
    renderWithClient(<ActivityImportCard activity={withSnapshot("toggl")} />);
    expect(screen.getByText("Snapshot taken 3 h ago.")).toBeInTheDocument();
    expect(screen.getByText("Not imported yet.")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Import Toggl activity" })).toBeEnabled();
  });

  it("shows the stored Activity, its date range and the last Import's counts", () => {
    renderWithClient(<ActivityImportCard activity={imported("clockify")} />);
    expect(screen.getByText(/10 Activity rows from 3 apps/)).toBeInTheDocument();
    expect(screen.getByText(/10 new, 1 skipped/)).toBeInTheDocument();
  });

  it.each([
    ["toggl", "Import Toggl activity"],
    ["clockify", "Import Clockify activity"],
  ] as const)("imports %s through its own route", async (source, button) => {
    const fetchMock = stubApi({
      [`POST /api/time/activity/${source}/import`]: () => jsonResponse(200, { read: 1, inserted: 1 }),
      "GET /api/time/status": () => jsonResponse(200, {}),
    });
    renderWithClient(<ActivityImportCard activity={withSnapshot(source)} />);

    await userEvent.click(screen.getByRole("button", { name: button }));

    await vi.waitFor(() => {
      const posts = fetchMock.mock.calls.map(([request]) => `${request.method} ${new URL(request.url).pathname}`);
      expect(posts).toContain(`POST /api/time/activity/${source}/import`);
    });
  });

  it("explains a failed Import", async () => {
    stubApi({
      "POST /api/time/activity/toggl/import": () =>
        jsonResponse(422, { status: 422, detail: "x", code: "snapshot_unreadable" }),
      "GET /api/time/status": () => jsonResponse(200, {}),
    });
    renderWithClient(<ActivityImportCard activity={withSnapshot("toggl")} />);

    await userEvent.click(screen.getByRole("button", { name: "Import Toggl activity" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(/could not be read/);
  });
});
