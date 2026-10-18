import { RequestError } from "../callApi";
import { toFinanceFailure } from "./failure";

const rejected = (code: string) => new RequestError("x", "rejected", 409, { code });

describe("toFinanceFailure", () => {
  it.each([
    "statement_not_readable",
    "new_account_required",
    "account_ambiguous",
    "statement_sections_changed",
    "confirm_in_progress",
  ])("reads the %s code", (code) => {
    expect(toFinanceFailure(rejected(code))).toEqual({ kind: code });
  });

  it("keeps the server's message for a code it does not know", () => {
    expect(toFinanceFailure(new RequestError("Name is empty.", "rejected", 400, { code: "empty_name" }))).toEqual({
      kind: "other",
      message: "Name is empty.",
    });
  });

  it("tells a down server from a failed reader", () => {
    expect(toFinanceFailure(new RequestError("x", "unreachable"))).toEqual({ kind: "server_unreachable" });
    expect(toFinanceFailure(new RequestError("x", "unreachable", 502))).toEqual({ kind: "reader_failed" });
  });

  it("falls back for anything that is not a request error", () => {
    expect(toFinanceFailure(new Error("boom")).kind).toBe("other");
  });
});
