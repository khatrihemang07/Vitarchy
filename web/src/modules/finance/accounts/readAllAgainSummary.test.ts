import type { ReadAgainItem } from "@/api/finance";
import { preview } from "../test-utils";
import { type ItemChanges, itemChangesLine, summariseReadAllAgain } from "./readAllAgainSummary";

function item(id: number, add: number, unchanged: number, lost: number): ReadAgainItem {
  return {
    statement_id: id,
    file_name: `s${String(id)}.pdf`,
    error_code: null,
    preview: {
      statement: preview({ id }),
      sections: [
        {
          position: 0,
          add_count: add,
          unchanged_count: unchanged,
          new_printing_count: 0,
          enriched_count: 0,
          no_longer_found_count: lost,
          added: [], new_printings: [],
        },
      ],
      payment_records: null,
    },
  };
}

describe("summariseReadAllAgain", () => {
  it("sums the counts and lists only the Statements with changes", () => {
    const summary = summariseReadAllAgain([item(1, 2, 10, 1), item(2, 0, 5, 0)]);
    expect(summary).toMatchObject({ add: 2, unchanged: 15, noLongerFound: 1 });
    expect(summary.changed.map((c) => c.statementId)).toEqual([1]);
  });

  it("sums Printings to add and rows that gain detail, and counts a Printings-only Statement as changed", () => {
    const only = item(4, 0, 3, 0);
    const section = only.preview?.sections[0];
    if (section) Object.assign(section, { new_printing_count: 5, enriched_count: 2 });
    const summary = summariseReadAllAgain([only]);
    expect(summary).toMatchObject({ newPrintings: 5, enriched: 2 });
    expect(summary.changed.map((c) => c.statementId)).toEqual([4]);
    expect(itemChangesLine(summary.changed[0] as ItemChanges)).toBe("0 to add, 5 Printings to add, 2 would gain detail, 0 no longer found");
  });

  it("counts a Payment history's new records as additions", () => {
    const history: ReadAgainItem = {
      ...item(3, 0, 0, 0),
      preview: { statement: preview({ id: 3 }), sections: [], payment_records: { add_count: 4, unchanged_count: 6 } },
    };
    expect(summariseReadAllAgain([history])).toMatchObject({ add: 4, unchanged: 6, noLongerFound: 0 });
  });

  it("lists a failed Statement with words and keeps it out of the totals", () => {
    const failed: ReadAgainItem = { statement_id: 9, file_name: "x.pdf", preview: null, error_code: "statement_not_readable" };
    const summary = summariseReadAllAgain([failed, item(1, 1, 1, 0)]);
    expect(summary.failed).toEqual([{ statementId: 9, fileName: "x.pdf", reason: expect.stringContaining("not read") as string }]);
    expect(summary.add).toBe(1);
  });

  it("is empty for no items", () => {
    expect(summariseReadAllAgain([])).toEqual({ changed: [], failed: [], add: 0, newPrintings: 0, enriched: 0, unchanged: 0, noLongerFound: 0 });
  });
});
