import { screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { jsonResponse, preview, renderWithClient, requestsOf, stubApi } from "../test-utils";
import { ReadAllAgain } from "./ReadAllAgain";

afterEach(() => vi.unstubAllGlobals());

function item(id: number, name: string, add: number, lost: number) {
  return {
    statement_id: id,
    file_name: name,
    error_code: null,
    preview: {
      statement: preview({ id, status: "confirmed" }),
      sections: [{ position: 0, add_count: add, unchanged_count: 10, no_longer_found_count: lost, added: [], new_printings: [] }],
    },
  };
}

describe("ReadAllAgain", () => {
  it("previews every Statement, then applies the ones with changes", async () => {
    const fetchMock = stubApi({
      "POST /api/finance/statements/read-again": () => jsonResponse(200, [item(1, "march.pdf", 2, 1), item(2, "april.pdf", 0, 0)]),
      "POST /api/finance/statements/read-again/confirm": () =>
        jsonResponse(200, { statements: 1, added: 2, no_longer_found: 1, failed: 0 }),
    });
    renderWithClient(<ReadAllAgain />);

    await userEvent.click(screen.getByRole("button", { name: "Read all again" }));

    expect(await screen.findByText(/1 of 2 Statements would change/)).toBeInTheDocument();
    expect(screen.getByText(/2 to add, 20 unchanged, 1 no longer found/)).toBeInTheDocument();
    const changed = screen.getByRole("list", { name: "Statements with changes" });
    expect(changed).toHaveTextContent("march.pdf");
    expect(changed).not.toHaveTextContent("april.pdf");

    await userEvent.click(screen.getByRole("button", { name: "Apply all changes" }));

    expect(await screen.findByText(/1 Statements changed: 2 added, 1 no longer found/)).toBeInTheDocument();
    expect(requestsOf(fetchMock)).toContain("POST /api/finance/statements/read-again/confirm");
  });

  it("offers no Apply when nothing has changed", async () => {
    stubApi({ "POST /api/finance/statements/read-again": () => jsonResponse(200, [item(1, "march.pdf", 0, 0)]) });
    renderWithClient(<ReadAllAgain />);

    await userEvent.click(screen.getByRole("button", { name: "Read all again" }));

    expect(await screen.findByText(/Nothing has changed/)).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Apply all changes" })).not.toBeInTheDocument();
  });

  it("names a Statement that could not be read again", async () => {
    stubApi({
      "POST /api/finance/statements/read-again": () =>
        jsonResponse(200, [{ statement_id: 5, file_name: "locked.pdf", preview: null, error_code: "statement_not_readable" }, item(1, "march.pdf", 1, 0)]),
    });
    renderWithClient(<ReadAllAgain />);

    await userEvent.click(screen.getByRole("button", { name: "Read all again" }));

    const failed = await screen.findByRole("list", { name: "Statements that could not be read again" });
    expect(failed).toHaveTextContent("locked.pdf");
  });

  it("says so when no Statement has its original stored", async () => {
    stubApi({ "POST /api/finance/statements/read-again": () => jsonResponse(200, []) });
    renderWithClient(<ReadAllAgain />);

    await userEvent.click(screen.getByRole("button", { name: "Read all again" }));

    expect(await screen.findByText(/nothing to read again/)).toBeInTheDocument();
  });

  it("explains a failed read and a failed apply, and closes with Cancel", async () => {
    stubApi({
      "POST /api/finance/statements/read-again": [
        () => jsonResponse(502, { status: 502, detail: "x", code: "reader_crashed" }),
        () => jsonResponse(200, [item(1, "march.pdf", 1, 0)]),
      ],
      "POST /api/finance/statements/read-again/confirm": () => jsonResponse(409, { status: 409, detail: "x", code: "confirm_in_progress" }),
    });
    renderWithClient(<ReadAllAgain />);

    await userEvent.click(screen.getByRole("button", { name: "Read all again" }));
    expect(await screen.findByText(/statement reader failed/)).toBeInTheDocument();

    await userEvent.click(screen.getByRole("button", { name: "Read all again" }));
    await userEvent.click(await screen.findByRole("button", { name: "Apply all changes" }));
    expect(await screen.findByText(/Another confirm is running/)).toBeInTheDocument();

    await userEvent.click(screen.getByRole("button", { name: "Cancel" }));
    expect(screen.queryByRole("button", { name: "Apply all changes" })).not.toBeInTheDocument();
  });
});
