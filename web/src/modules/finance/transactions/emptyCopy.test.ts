import { emptyCopy, emptyReason } from "./emptyCopy";

describe("emptyReason", () => {
  it("blames the filters when there are any, whatever is stored", () => {
    expect(emptyReason({ filtered: true, everStored: true })).toBe("filtered");
    expect(emptyReason({ filtered: true, everStored: false })).toBe("filtered");
  });
  it("blames the Period when Transactions exist elsewhere", () => {
    expect(emptyReason({ filtered: false, everStored: true })).toBe("empty-period");
  });
  it("says nothing is stored when nothing is", () => {
    expect(emptyReason({ filtered: false, everStored: false })).toBe("nothing-stored");
  });
});

describe("emptyCopy", () => {
  it("has words for every reason", () => {
    expect(emptyCopy("filtered").title).toBe("No Transactions match these filters");
    expect(emptyCopy("empty-period").title).toBe("No Transactions in this Period");
    expect(emptyCopy("nothing-stored").title).toBe("No Transactions yet");
  });
});
