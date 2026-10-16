import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { ConnectForm } from "./ConnectForm";
import { jsonResponse, renderWithClient, stubApi } from "./test-utils";

afterEach(() => vi.unstubAllGlobals());

describe("ConnectForm", () => {
  it("hides the token and keeps Connect off until one is typed", async () => {
    renderWithClient(<ConnectForm />);

    const input = screen.getByLabelText("Toggl API token");
    expect(input).toHaveAttribute("type", "password");
    expect(screen.getByRole("button", { name: "Connect" })).toBeDisabled();
    await userEvent.type(input, "abc");
    expect(screen.getByRole("button", { name: "Connect" })).toBeEnabled();
  });

  it("sends the trimmed token once and clears the field", async () => {
    const bodies: unknown[] = [];
    stubApi({
      "PUT /api/time/connection": async (request) => {
        bodies.push(await request.json());
        return jsonResponse(200, { workspace_name: "My Workspace" });
      },
      "GET /api/time/status": () => jsonResponse(200, {}),
    });
    renderWithClient(<ConnectForm />);

    await userEvent.type(screen.getByLabelText("Toggl API token"), "  secret-token ");
    await userEvent.click(screen.getByRole("button", { name: "Connect" }));

    await vi.waitFor(() => {
      expect(screen.getByLabelText("Toggl API token")).toHaveValue("");
    });
    expect(bodies).toEqual([{ api_token: "secret-token" }]);
  });

  it("explains a rejected token and keeps what was typed", async () => {
    stubApi({
      "PUT /api/time/connection": () =>
        jsonResponse(401, { status: 401, detail: "Toggl rejected this API token.", code: "invalid_token" }),
    });
    renderWithClient(<ConnectForm />);

    await userEvent.type(screen.getByLabelText("Toggl API token"), "bad");
    await userEvent.click(screen.getByRole("button", { name: "Connect" }));

    expect(await screen.findByRole("alert")).toHaveTextContent("rejected this API token");
    expect(screen.getByLabelText("Toggl API token")).toHaveValue("bad");
  });
});
