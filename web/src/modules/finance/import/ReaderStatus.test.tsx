import { screen } from "@testing-library/react";
import { jsonResponse, renderWithClient, stubApi } from "../test-utils";
import { ReaderStatus } from "./ReaderStatus";

afterEach(() => vi.unstubAllGlobals());

const status = (reader_available: boolean) => () =>
  jsonResponse(200, { account_count: 0, unconfirmed_statement_count: 0, unlinked_payment_record_count: 0, reader_available, latest_runs: [] });

describe("ReaderStatus", () => {
  it("says the reader is ready", async () => {
    stubApi({ "GET /api/finance/status": status(true) });
    renderWithClient(<ReaderStatus />);
    expect(await screen.findByText("The statement reader is ready.")).toBeInTheDocument();
  });

  it("tells the person how to install a missing reader", async () => {
    stubApi({ "GET /api/finance/status": status(false) });
    renderWithClient(<ReaderStatus />);
    expect(await screen.findByText("make finance-reader-install")).toBeInTheDocument();
    expect(screen.getByText("The statement reader is not installed")).toBeInTheDocument();
  });

  it("shows a loading state, then a failure that can be retried", async () => {
    stubApi({ "GET /api/finance/status": () => jsonResponse(500, {}) });
    renderWithClient(<ReaderStatus />);
    expect(screen.getByRole("status")).toHaveTextContent("Checking the statement reader");
    expect(await screen.findByRole("button", { name: "Try again" })).toBeInTheDocument();
  });
});
