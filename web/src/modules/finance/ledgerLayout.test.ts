import { readFileSync } from "node:fs";
import { STATEMENT_ROW_GRID, STATEMENT_ROW_HEADINGS, TRANSACTION_GRID, TRANSACTION_HEADINGS } from "./ledgerLayout";

const BASE_CSS = readFileSync("src/design/theme/base.css", "utf8");

/** Counts the tracks the stylesheet defines for a `grid-cols-ledger*` utility. */
function trackCount(utility: string): number {
  const rule = new RegExp(`@utility ${utility} \\{\\s*grid-template-columns: ([^;]+);`).exec(BASE_CSS)?.[1] ?? "";
  return rule.replace(/,\s+/g, ",").split(/\s+(?![^(]*\))/).length;
}

describe("ledger layout", () => {
  it("has one heading per grid track in the Transactions page", () => {
    expect(TRANSACTION_GRID).toContain("md:grid-cols-ledger");
    expect(trackCount("grid-cols-ledger")).toBe(TRANSACTION_HEADINGS.length);
  });

  it("has one heading per grid track in a Statement's rows", () => {
    expect(STATEMENT_ROW_GRID).toContain("md:grid-cols-ledger-statement");
    expect(trackCount("grid-cols-ledger-statement")).toBe(STATEMENT_ROW_HEADINGS.length);
  });

  it("takes every width from the --track tokens, none written here", () => {
    expect(BASE_CSS.match(/@utility grid-cols-ledger[^{]*\{[^}]*\}/g)?.join("")).not.toMatch(/\d+(\.\d+)?rem/);
  });

  it("lines numbers up to the end", () => {
    const aligned = TRANSACTION_HEADINGS.filter((heading) => heading.align === "end").map((heading) => heading.id);
    expect(aligned).toEqual(["amount", "balance"]);
  });
});
