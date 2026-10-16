import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { ConnectedView } from "./ConnectedView";
import { jsonResponse, renderWithClient, stubApi } from "./test-utils";

afterEach(() => vi.unstubAllGlobals());

describe("ConnectedView", () => {
  it("names the workspace and says synced entries are kept", () => {
    renderWithClient(<ConnectedView workspaceName="My Workspace" />);

    expect(screen.getByText("My Workspace")).toBeInTheDocument();
    expect(screen.getByText(/already synced are kept/)).toBeInTheDocument();
  });

  it("disconnects when asked", async () => {
    const fetchMock = stubApi({
      "DELETE /api/time/connection": () => new Response(null, { status: 204 }),
      "GET /api/time/status": () => jsonResponse(200, {}),
    });
    renderWithClient(<ConnectedView workspaceName="My Workspace" />);

    await userEvent.click(screen.getByRole("button", { name: "Disconnect" }));

    await vi.waitFor(() => {
      expect(fetchMock.mock.calls.some(([request]) => request.method === "DELETE")).toBe(true);
    });
  });

  it("shows why disconnecting failed", async () => {
    stubApi({ "DELETE /api/time/connection": () => new Response("", { status: 502 }) });
    renderWithClient(<ConnectedView workspaceName="My Workspace" />);

    await userEvent.click(screen.getByRole("button", { name: "Disconnect" }));

    expect(await screen.findByRole("alert")).toHaveTextContent("Could not reach the Vitarchy server.");
  });
});
