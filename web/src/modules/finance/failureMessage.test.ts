import { RequestError } from "@/api/callApi";
import { errorMessage } from "./failureMessage";

describe("errorMessage", () => {
  it("explains an unrecognised Account in the words of the Glossary", () => {
    const error = new RequestError("x", "rejected", 400, { code: "new_account_required" });
    expect(errorMessage(error)).toMatch(/New Account/);
  });

  it("explains a Read again that found different Accounts", () => {
    const error = new RequestError("x", "rejected", 409, { code: "statement_sections_changed" });
    expect(errorMessage(error)).toMatch(/different Accounts/);
  });

  it("falls back to the server's message", () => {
    expect(errorMessage(new RequestError("The name cannot be empty.", "rejected", 400))).toBe("The name cannot be empty.");
  });
});
