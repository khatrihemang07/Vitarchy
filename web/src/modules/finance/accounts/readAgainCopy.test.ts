import { appliedLine, sectionChangesLine } from "./readAgainCopy";

describe("readAgainCopy", () => {
  it("describes a section's Read again with Printings and detail", () => {
    const changes = { position: 0, add_count: 1, added: [], new_printings: [], enriched_count: 2, new_printing_count: 3, unchanged_count: 10, no_longer_found_count: 0 };
    expect(sectionChangesLine(changes)).toBe(
      "1 to add, 3 Printings to add to stored Transactions, 2 would gain detail, 10 unchanged, 0 no longer found (marked, never deleted).",
    );
    expect(sectionChangesLine({ ...changes, new_printing_count: 0, enriched_count: 0 })).toBe(
      "1 to add, 10 unchanged, 0 no longer found (marked, never deleted).",
    );
  });

  it("describes an applied Read again and Read all again", () => {
    const one = { added: 1, enriched: 1, linked: 0, no_longer_found: 2, printings_added: 4, unchanged: 7 };
    expect(appliedLine(one)).toBe("1 added, 4 Printings added to stored Transactions, 1 gained detail, 7 unchanged, 2 no longer found");
    const all = { added: 0, enriched: 0, failed: 0, linked: 0, no_longer_found: 0, printings_added: 0, statements: 1 };
    expect(appliedLine(all)).toBe("0 added, 0 no longer found");
  });
});
