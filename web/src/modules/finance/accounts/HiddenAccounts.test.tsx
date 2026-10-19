import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { account, jsonResponse, renderWithClient, requestsOf, stubApi } from "../test-utils";
import { AccountCard } from "./AccountCard";
import { HiddenAccounts } from "./HiddenAccounts";

afterEach(() => vi.unstubAllGlobals());

describe("hiding Accounts", () => {
  it("shows nothing when no Account is hidden", () => {
    const { container } = renderWithClient(<HiddenAccounts accounts={[]} />);
    expect(container).toBeEmptyDOMElement();
  });

  it("keeps hidden Accounts behind a collapsed Hidden heading until opened", async () => {
    renderWithClient(<HiddenAccounts accounts={[account({ id: 7, name: "Old loan", hidden: true })]} />);
    const toggle = screen.getByRole("button", { name: "Hidden (1)" });
    expect(toggle).toHaveAttribute("aria-expanded", "false");
    expect(screen.queryByText("Old loan")).not.toBeInTheDocument();

    await userEvent.click(toggle);

    expect(toggle).toHaveAttribute("aria-expanded", "true");
    expect(await screen.findByText("Old loan")).toBeInTheDocument();
  });

  it("hides an Account from its card", async () => {
    const fetchMock = stubApi({ "PATCH /api/finance/accounts/1": () => jsonResponse(200, account({ hidden: true })) });
    renderWithClient(<AccountCard account={account()} />);

    await userEvent.click(screen.getByRole("button", { name: "Hide HDFC Savings" }));

    await vi.waitFor(() => { expect(requestsOf(fetchMock)).toContain("PATCH /api/finance/accounts/1"); });
    const patch = fetchMock.mock.calls.map(([r]) => r).find((r) => r.method === "PATCH");
    expect(await patch?.json()).toEqual({ hidden: true });
  });

  it("offers Unhide on a hidden Account and marks it Hidden", async () => {
    const fetchMock = stubApi({ "PATCH /api/finance/accounts/1": () => jsonResponse(200, account()) });
    renderWithClient(<AccountCard account={account({ hidden: true })} />);
    expect(screen.getByText("Hidden")).toBeInTheDocument();

    await userEvent.click(screen.getByRole("button", { name: "Unhide HDFC Savings" }));

    await vi.waitFor(() => { expect(requestsOf(fetchMock)).toContain("PATCH /api/finance/accounts/1"); });
    const patch = fetchMock.mock.calls.map(([r]) => r).find((r) => r.method === "PATCH");
    expect(await patch?.json()).toEqual({ hidden: false });
  });
});
