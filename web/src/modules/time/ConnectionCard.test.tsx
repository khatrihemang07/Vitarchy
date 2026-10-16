import { screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { ConnectionCard } from "./ConnectionCard";
import { disconnected, jsonResponse, renderWithClient, stubApi, synced } from "./test-utils";

afterEach(() => vi.unstubAllGlobals());

describe("ConnectionCard", () => {
  it("shows the token form when not connected", async () => {
    stubApi({ "GET /api/time/status": () => jsonResponse(200, disconnected) });
    renderWithClient(<ConnectionCard />);
    expect(await screen.findByLabelText("Toggl API token")).toBeInTheDocument();
  });

  it("shows a loading state while the status is pending", () => {
    stubApi({ "GET /api/time/status": () => new Promise<Response>(() => undefined) });
    renderWithClient(<ConnectionCard />);
    expect(screen.getByText("Checking connection…")).toBeInTheDocument();
  });

  it("shows an error when the server cannot be reached", async () => {
    stubApi({ "GET /api/time/status": () => new Response("", { status: 502 }) });
    renderWithClient(<ConnectionCard />);
    expect(await screen.findByRole("alert")).toHaveTextContent("Could not reach the Vitarchy server.");
  });

  it("sends the token, clears the field and shows the workspace once connected", async () => {
    let connected = false;
    const fetchMock = stubApi({
      "GET /api/time/status": () => jsonResponse(200, connected ? synced : disconnected),
      "PUT /api/time/connection": async (request) => {
        expect(await request.json()).toEqual({ api_token: "secret-token" });
        connected = true;
        return jsonResponse(200, { workspace_name: "My Workspace" });
      },
    });
    renderWithClient(<ConnectionCard />);

    await userEvent.type(await screen.findByLabelText("Toggl API token"), "secret-token");
    await userEvent.click(screen.getByRole("button", { name: "Connect" }));

    expect(await screen.findByText("My Workspace")).toBeInTheDocument();
    expect(screen.queryByLabelText("Toggl API token")).not.toBeInTheDocument();
    expect(fetchMock).toHaveBeenCalled();
  });

  it("shows the workspace when connected, and disconnecting returns to the form", async () => {
    let connected = true;
    stubApi({
      "GET /api/time/status": () => jsonResponse(200, connected ? synced : disconnected),
      "DELETE /api/time/connection": () => {
        connected = false;
        return new Response(null, { status: 204 });
      },
    });
    renderWithClient(<ConnectionCard />);

    expect(await screen.findByText("My Workspace")).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Disconnect" }));

    await waitFor(() => {
      expect(screen.getByLabelText("Toggl API token")).toBeInTheDocument();
    });
  });
});
