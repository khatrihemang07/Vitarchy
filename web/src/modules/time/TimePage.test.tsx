import { screen } from "@testing-library/react";
import { TimePage } from "./TimePage";
import { disconnected, jsonResponse, renderWithClient, stubApi } from "./test-utils";

afterEach(() => vi.unstubAllGlobals());

describe("TimePage", () => {
  it("shows one skeleton for the whole page while the status is pending", () => {
    stubApi({ "GET /api/time/status": () => new Promise<Response>(() => undefined) });
    renderWithClient(<TimePage />);
    expect(screen.getAllByRole("status")).toHaveLength(2);
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });

  it("shows one failure, with Try again, when the status cannot be loaded", async () => {
    stubApi({ "GET /api/time/status": () => new Response("", { status: 502 }) });
    renderWithClient(<TimePage />);
    expect(await screen.findAllByRole("alert")).toHaveLength(1);
    expect(screen.getByRole("button", { name: "Try again" })).toBeInTheDocument();
  });

  it("shows the connection, sync, import and Timeline cards once the status is in", async () => {
    stubApi({ "GET /api/time/status": () => jsonResponse(200, disconnected) });
    renderWithClient(<TimePage />);
    expect(await screen.findByText("Toggl connection")).toBeInTheDocument();
    expect(screen.getByText("Sync from Toggl")).toBeInTheDocument();
    expect(screen.getByText("Timeline")).toBeInTheDocument();
  });
});
