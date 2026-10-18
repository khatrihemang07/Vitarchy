import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { jsonResponse, renderWithClient, requestsOf, stubApi } from "../test-utils";
import { PasswordsCard } from "./PasswordsCard";

afterEach(() => vi.unstubAllGlobals());

const saved = { id: 4, label: "HDFC card", created_at: "2026-05-01T10:00:00Z", last_used_at: "2026-05-03T08:00:00Z" };

describe("PasswordsCard", () => {
  it("lists labels with when each last opened a file, never a secret", async () => {
    stubApi({ "GET /api/finance/statement-passwords": () => jsonResponse(200, [saved, { ...saved, id: 5, label: "Spare", last_used_at: null }]) });
    renderWithClient(<PasswordsCard />);
    expect(await screen.findByText("HDFC card")).toBeInTheDocument();
    expect(screen.getByText("Last used 3 May 2026")).toBeInTheDocument();
    expect(screen.getByText("Never used")).toBeInTheDocument();
  });

  it("says so when none are saved", async () => {
    stubApi({ "GET /api/finance/statement-passwords": () => jsonResponse(200, []) });
    renderWithClient(<PasswordsCard />);
    expect(await screen.findByText("No Statement passwords saved yet.")).toBeInTheDocument();
  });

  it("explains a list that failed to load", async () => {
    stubApi({ "GET /api/finance/statement-passwords": () => jsonResponse(500, {}) });
    renderWithClient(<PasswordsCard />);
    expect(await screen.findByRole("alert")).toBeInTheDocument();
  });

  it("hides the password as it is typed, sends it once and clears the field", async () => {
    const fetchMock = stubApi({
      "GET /api/finance/statement-passwords": () => jsonResponse(200, []),
      "POST /api/finance/statement-passwords": () => jsonResponse(201, saved),
    });
    renderWithClient(<PasswordsCard />);
    const secret = screen.getByLabelText("Password");
    expect(secret).toHaveAttribute("type", "password");

    await userEvent.type(screen.getByLabelText("Label"), "HDFC card");
    await userEvent.type(secret, "hunter2");
    await userEvent.click(screen.getByRole("button", { name: "Save password" }));

    await vi.waitFor(() => { expect(requestsOf(fetchMock)).toContain("POST /api/finance/statement-passwords"); });
    const post = fetchMock.mock.calls.map(([r]) => r).find((r) => r.method === "POST");
    expect(await post?.json()).toEqual({ label: "HDFC card", secret: "hunter2" });
    expect(secret).toHaveValue("");
    expect(screen.queryByDisplayValue("hunter2")).not.toBeInTheDocument();
  });

  it("asks before removing a password, and keeps it when the person says Keep", async () => {
    const fetchMock = stubApi({
      "GET /api/finance/statement-passwords": () => jsonResponse(200, [saved]),
      "DELETE /api/finance/statement-passwords/4": () => new Response(null, { status: 204 }),
    });
    renderWithClient(<PasswordsCard />);

    await userEvent.click(await screen.findByRole("button", { name: "Remove HDFC card" }));
    expect(screen.getByText("Remove this password?")).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Keep" }));
    expect(requestsOf(fetchMock)).not.toContain("DELETE /api/finance/statement-passwords/4");

    await userEvent.click(screen.getByRole("button", { name: "Remove HDFC card" }));
    await userEvent.click(screen.getByRole("button", { name: "Remove" }));
    await vi.waitFor(() => { expect(requestsOf(fetchMock)).toContain("DELETE /api/finance/statement-passwords/4"); });
  });
});
