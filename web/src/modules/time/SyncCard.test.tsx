import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { SyncCard } from "./SyncCard";
import { disconnected, jsonResponse, renderWithClient, stubApi, synced } from "./test-utils";

afterEach(() => vi.unstubAllGlobals());

const connectedNeverSynced = { ...synced, last_sync: null, time_entry_count: 0 };

describe("SyncCard", () => {
  it("shows a loading state while the status is pending", () => {
    stubApi({ "GET /api/time/status": () => new Promise<Response>(() => undefined) });
    renderWithClient(<SyncCard />);
    expect(screen.getByText("Loading status…")).toBeInTheDocument();
  });

  it("shows an error when the status cannot be loaded", async () => {
    stubApi({ "GET /api/time/status": () => new Response("", { status: 502 }) });
    renderWithClient(<SyncCard />);
    expect(await screen.findByRole("alert")).toBeInTheDocument();
  });

  it("disables Sync and says why when Toggl is not connected", async () => {
    stubApi({ "GET /api/time/status": () => jsonResponse(200, disconnected) });
    renderWithClient(<SyncCard />);
    expect(await screen.findByText("Connect Toggl to start syncing.")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Sync now" })).toBeDisabled();
  });

  it("invites a first Sync when connected and never synced", async () => {
    stubApi({ "GET /api/time/status": () => jsonResponse(200, connectedNeverSynced) });
    renderWithClient(<SyncCard />);
    expect(await screen.findByText("Not synced yet.")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Sync now" })).toBeEnabled();
  });

  it("shows the last run's result and the mirrored span", async () => {
    stubApi({ "GET /api/time/status": () => jsonResponse(200, synced) });
    renderWithClient(<SyncCard />);
    expect(await screen.findByText(/2 new/)).toBeInTheDocument();
    expect(screen.getByText(/2 Time Entries/)).toBeInTheDocument();
  });

  it("refreshes the status after a Sync", async () => {
    stubApi({
      "GET /api/time/status": [() => jsonResponse(200, connectedNeverSynced), () => jsonResponse(200, synced)],
      "POST /api/time/sync": () => jsonResponse(200, { inserted: 2 }),
    });
    renderWithClient(<SyncCard />);

    await userEvent.click(await screen.findByRole("button", { name: "Sync now" }));

    expect(await screen.findByText(/2 new/)).toBeInTheDocument();
  });

  it("says when Toggl's rate limit resets", async () => {
    stubApi({
      "GET /api/time/status": () => jsonResponse(200, synced),
      "POST /api/time/sync": () =>
        jsonResponse(429, { status: 429, detail: "limit", code: "rate_limited", resets_in_secs: 1800 }),
    });
    renderWithClient(<SyncCard />);

    await userEvent.click(await screen.findByRole("button", { name: "Sync now" }));

    expect(await screen.findByRole("alert")).toHaveTextContent("It resets in 30 min.");
  });

  it("tells the person to reconnect when the token was rejected", async () => {
    stubApi({
      "GET /api/time/status": () => jsonResponse(200, synced),
      "POST /api/time/sync": () => jsonResponse(401, { status: 401, detail: "x", code: "invalid_token" }),
    });
    renderWithClient(<SyncCard />);

    await userEvent.click(await screen.findByRole("button", { name: "Sync now" }));

    expect(await screen.findByRole("alert")).toHaveTextContent("rejected this API token");
  });
});
