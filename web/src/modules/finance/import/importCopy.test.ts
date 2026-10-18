import { confirmedLine, previewCountLine } from "./importCopy";

describe("importCopy", () => {
  it("adds 'would gain detail' to the preview line only when some rows would", () => {
    const base = { new_count: 3, already_stored_count: 1, enriched_count: 0 };
    expect(previewCountLine(base)).toEqual({ lead: "3 new", rest: ", 1 already stored" });
    expect(previewCountLine({ ...base, enriched_count: 2 }).rest).toBe(", 1 already stored, 2 would gain detail");
  });


  it("says what a confirm did, leaving out zero extras", () => {
    expect(confirmedLine({ added: 2, already_stored: 1, enriched: 0 })).toBe("2 added, 1 already stored.");
    expect(confirmedLine({ added: 2, already_stored: 4, enriched: 3 })).toBe(
      "2 added, 4 already stored, 3 gained detail.",
    );
  });
});
