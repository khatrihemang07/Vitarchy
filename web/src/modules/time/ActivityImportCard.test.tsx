import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import type { TimeStatus } from "@/api/time";

import { ActivityImportCard } from "./ActivityImportCard";
import { disconnected, jsonResponse, renderWithClient, stubApi } from "./test-utils";

afterEach(() => vi.unstubAllGlobals());

const withSnapshot: TimeStatus = {
  ...disconnected,
  activity: { activity_count: 0, app_count: 0, snapshot_taken_at: new Date(Date.now() - 3 * 3_600_000).toISOString() },
};

const imported: TimeStatus = {
  ...disconnected,
  activity: {
    ...withSnapshot.activity,
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
  },
};

describe("ActivityImportCard", () => {
  it("shows a loading state while the status is pending", () => {
    stubApi({ "GET /api/time/status": () => new Promise<Response>(() => undefined) });
    renderWithClient(<ActivityImportCard />);
    expect(screen.getByText("Loading status…")).toBeInTheDocument();
  });

  it("shows an error when the status cannot be loaded", async () => {
    stubApi({ "GET /api/time/status": () => new Response("", { status: 502 }) });
    renderWithClient(<ActivityImportCard />);
    expect(await screen.findByRole("alert")).toBeInTheDocument();
  });

  it("tells the person to install the helper when there is no Snapshot, and disables Import", async () => {
    stubApi({ "GET /api/time/status": () => jsonResponse(200, disconnected) });
    renderWithClient(<ActivityImportCard />);
    expect(await screen.findByText(/No Snapshot yet/)).toBeInTheDocument();
    expect(screen.getByText("make helper-install")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Import activity" })).toBeDisabled();
  });

  it("shows the Snapshot's age and invites a first Import", async () => {
    stubApi({ "GET /api/time/status": () => jsonResponse(200, withSnapshot) });
    renderWithClient(<ActivityImportCard />);
    expect(await screen.findByText("Snapshot taken 3 h ago.")).toBeInTheDocument();
    expect(screen.getByText("Not imported yet.")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Import activity" })).toBeEnabled();
  });

  it("shows the stored Activity, its date range and the last Import's counts", async () => {
    stubApi({ "GET /api/time/status": () => jsonResponse(200, imported) });
    renderWithClient(<ActivityImportCard />);
    expect(await screen.findByText(/10 Activity rows from 3 apps/)).toBeInTheDocument();
    expect(screen.getByText(/10 new, 1 skipped/)).toBeInTheDocument();
  });

  it("refreshes the status after an Import", async () => {
    stubApi({
      "GET /api/time/status": [() => jsonResponse(200, withSnapshot), () => jsonResponse(200, imported)],
      "POST /api/time/activity/import": () => jsonResponse(200, { read: 11, inserted: 10, skipped: 1 }),
    });
    renderWithClient(<ActivityImportCard />);

    await userEvent.click(await screen.findByRole("button", { name: "Import activity" }));

    expect(await screen.findByText(/10 new, 1 skipped/)).toBeInTheDocument();
  });

  it("explains a failed Import", async () => {
    stubApi({
      "GET /api/time/status": () => jsonResponse(200, withSnapshot),
      "POST /api/time/activity/import": () =>
        jsonResponse(422, { status: 422, detail: "x", code: "snapshot_unreadable" }),
    });
    renderWithClient(<ActivityImportCard />);

    await userEvent.click(await screen.findByRole("button", { name: "Import activity" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(/could not be read/);
  });
});
