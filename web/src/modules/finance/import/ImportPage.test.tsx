import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { jsonResponse, pendingForever, preview, renderWithClient, SETTLED_NOTICES, stubApi } from "../test-utils";
import { ImportPage } from "./ImportPage";

afterEach(() => vi.unstubAllGlobals());

const statusOk = () =>
  jsonResponse(200, { account_count: 0, unconfirmed_statement_count: 0, reader_available: true, latest_runs: [] });
const base = {
  "GET /api/finance/status": statusOk,
  "GET /api/finance/statement-passwords": () => jsonResponse(200, []),
  "GET /api/finance/accounts": () => jsonResponse(200, []),
};
const summary = {
  id: 9, file_name: "earlier.csv", format: "csv", kind: "statement", outcome: "read", status: "unconfirmed",
  uploaded_at: "2026-05-01T10:00:00Z", confirmed_at: null, account_count: 1, row_count: 12,
};

describe("ImportPage", () => {
  it("shows loading notices only, and no warning, empty or failure notice, while everything is pending", () => {
    stubApi({
      "GET /api/finance/status": pendingForever,
      "GET /api/finance/statements": pendingForever,
      "GET /api/finance/statement-passwords": pendingForever,
      "GET /api/finance/accounts": pendingForever,
    });
    const { container } = renderWithClient(<ImportPage />);
    expect(screen.getAllByText("Loading the Import page").length).toBeGreaterThan(0);
    expect(container.querySelectorAll(SETTLED_NOTICES)).toHaveLength(0);
  });

  it("warns once the status settles as reader not available", async () => {
    stubApi({
      ...base,
      "GET /api/finance/status": () =>
        jsonResponse(200, { account_count: 0, unconfirmed_statement_count: 0, reader_available: false, latest_runs: [] }),
      "GET /api/finance/statements": () => jsonResponse(200, []),
    });
    renderWithClient(<ImportPage />);
    expect(await screen.findByText("The statement reader is not installed")).toBeInTheDocument();
  });

  it("lists Statements waiting from earlier sessions", async () => {
    stubApi({ ...base, "GET /api/finance/statements": () => jsonResponse(200, [summary]) });
    renderWithClient(<ImportPage />);
    expect(await screen.findByText("earlier.csv")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Review" })).toBeInTheDocument();
  });

  it("says nothing is waiting when the list is empty", async () => {
    stubApi({ ...base, "GET /api/finance/statements": () => jsonResponse(200, []) });
    renderWithClient(<ImportPage />);
    expect(await screen.findByText("Nothing is waiting to be confirmed.")).toBeInTheDocument();
  });

  it("uploads several files at once and shows a preview card for each", async () => {
    const fetchMock = stubApi({
      ...base,
      "GET /api/finance/statements": () => jsonResponse(200, []),
      "POST /api/finance/statements": () =>
        jsonResponse(200, [preview({ id: 1, file_name: "a.pdf" }), preview({ id: 2, file_name: "b.csv", format: "csv" })]),
    });
    renderWithClient(<ImportPage />);

    await userEvent.upload(await screen.findByLabelText("Statement files"), [
      new File(["x"], "a.pdf", { type: "application/pdf" }),
      new File(["y"], "b.csv", { type: "text/csv" }),
    ]);

    expect(await screen.findByText("a.pdf")).toBeInTheDocument();
    expect(screen.getByText("b.csv")).toBeInTheDocument();
    expect(fetchMock.mock.calls.some(([r]) => r.method === "POST" && r.url.endsWith("/statements"))).toBe(true);
  });

  it("explains an upload the server refused", async () => {
    stubApi({
      ...base,
      "GET /api/finance/statements": () => jsonResponse(200, []),
      "POST /api/finance/statements": () => jsonResponse(413, { status: 413, detail: "x", code: "upload_too_large" }),
    });
    renderWithClient(<ImportPage />);
    await userEvent.upload(await screen.findByLabelText("Statement files"), new File(["x"], "a.pdf"));
    expect(await screen.findByText(/upload is too large/)).toBeInTheDocument();
  });
});

describe("ImportPage reviewing a waiting Statement", () => {
  it("moves it up as a card that keeps its result after it leaves the waiting list", async () => {
    stubApi({
      ...base,
      "GET /api/finance/statements": [() => jsonResponse(200, [summary]), () => jsonResponse(200, [])],
      "GET /api/finance/statements/9": () => jsonResponse(200, preview({ id: 9, file_name: "earlier.csv", sections: [], kind: "payment_history" })),
      "POST /api/finance/statements/9/confirm": () => jsonResponse(200, { added: 2, already_stored: 0, enriched: 0 }),
    });
    renderWithClient(<ImportPage />);

    await userEvent.click(await screen.findByRole("button", { name: "Review" }));
    await userEvent.click(await screen.findByRole("button", { name: "Confirm" }));

    expect(await screen.findByText(/2 added/)).toBeInTheDocument();
    expect(screen.getByText("earlier.csv")).toBeInTheDocument();
  });
});
