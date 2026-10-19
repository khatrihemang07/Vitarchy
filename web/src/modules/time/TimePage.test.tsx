import { screen } from "@testing-library/react";
import { TimePage } from "./TimePage";
import { disconnected, jsonResponse, renderWithClient, stubApi } from "./test-utils";

afterEach(() => vi.unstubAllGlobals());

describe("TimePage", () => {
  it("shows every card at once, each with its own loading notice, before the status resolves", () => {
    stubApi({ "GET /api/time/status": () => new Promise<Response>(() => undefined) });
    renderWithClient(<TimePage />);
    expect(screen.getByText("Toggl connection")).toBeInTheDocument();
    expect(screen.getByText("Sync from Toggl")).toBeInTheDocument();
    expect(screen.getByText("Checking connection…")).toBeInTheDocument();
    expect(screen.getAllByText("Loading status…")).toHaveLength(2);
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });

  it("shows the failure in each card that reads the status, with Try again", async () => {
    stubApi({ "GET /api/time/status": () => new Response("", { status: 502 }) });
    renderWithClient(<TimePage />);
    expect(await screen.findAllByRole("alert")).toHaveLength(3);
    expect(screen.getAllByRole("button", { name: "Try again" })).toHaveLength(3);
  });

  it("shows the connection, sync, import and Timeline cards once the status is in", async () => {
    stubApi({ "GET /api/time/status": () => jsonResponse(200, disconnected) });
    renderWithClient(<TimePage />);
    expect(await screen.findByLabelText("Toggl API token")).toBeInTheDocument();
    expect(screen.getByText("Sync from Toggl")).toBeInTheDocument();
    expect(await screen.findByText("Timeline")).toBeInTheDocument();
  });
});
