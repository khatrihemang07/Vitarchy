import { sectionHeading, accountKindLabel, checkCopy, hasFailedCheck, statementKindLabel, unreadCopy } from "./statementCopy";

const diagnostics = (stage: string | null = null) => ({ stage, code: null, pages: null, rows: null });

describe("unreadCopy", () => {
  it("says nothing for a Statement that was read", () => {
    expect(unreadCopy({ outcome: "read", diagnostics: diagnostics() })).toBeNull();
  });
  it("tells a locked file apart: no saved password opened it", () => {
    expect(unreadCopy({ outcome: "locked", diagnostics: diagnostics() })?.title).toBe("No saved password opened this file");
  });
  it("names the stage where a table was not found", () => {
    expect(unreadCopy({ outcome: "no_table", diagnostics: diagnostics("locate") })?.body).toMatch(/finding the table of Transactions/);
  });
  it("explains an unsupported file in plain words", () => {
    expect(unreadCopy({ outcome: "unsupported", diagnostics: diagnostics() })?.body).toMatch(/PDF, CSV, XLS or XLSX/);
  });
  it("treats any other outcome as a reader failure", () => {
    expect(unreadCopy({ outcome: "crashed", diagnostics: diagnostics() })?.title).toMatch(/failed/);
  });
});

describe("checkCopy", () => {
  it("words each outcome of each Check", () => {
    expect(checkCopy({ name: "balance_chain", outcome: "passed" })).toEqual({
      name: "Balances",
      text: "Each balance follows from the one before it.",
    });
    expect(checkCopy({ name: "summary_totals", outcome: "failed" }).text).toMatch(/differ/);
    expect(checkCopy({ name: "row_counts", outcome: "skipped" }).text).toMatch(/No row count/);
  });
  it("keeps an unknown Check readable", () => {
    expect(checkCopy({ name: "new_check", outcome: "passed" })).toEqual({ name: "new_check", text: "passed" });
  });
});

describe("labels", () => {
  it("uses the Glossary words", () => {
    expect(accountKindLabel("credit_card")).toBe("Credit card");
    expect(statementKindLabel("payment_history")).toBe("Payment history");
    expect(statementKindLabel("statement")).toBe("Statement");
  });
  it("flags a failed Check", () => {
    expect(hasFailedCheck([{ outcome: "passed" }, { outcome: "failed" }])).toBe(true);
    expect(hasFailedCheck([{ outcome: "skipped" }])).toBe(false);
  });
});

describe("sectionHeading", () => {
  it("names a Payment history section by its Instrument, with the issuer when known", () => {
    expect(sectionHeading("payment_history", { issuer: null, kind: null, mask: "icici r" })).toBe("Instrument: ICICI R");
    expect(sectionHeading("payment_history", { issuer: "HDFC", kind: null, mask: "xx1234" })).toBe("Instrument: XX1234 · HDFC");
  });
  it("names a bank Statement section by issuer, kind and number", () => {
    expect(sectionHeading("statement", { issuer: "SBI", kind: "loan", mask: "xx98" })).toBe("SBI · Loan · XX98");
  });
});
