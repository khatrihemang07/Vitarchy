import { printedInLabel } from "./printedInLabel";

describe("printedInLabel", () => {
  it("names how many Statements printed a Transaction", () => {
    expect(printedInLabel(2)).toBe("Printed in 2 Statements");
  });
});
