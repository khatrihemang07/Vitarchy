import { account } from "../test-utils";
import { splitHidden } from "./hiddenGroup";

describe("splitHidden", () => {
  it("keeps the order inside each group", () => {
    const all = [account({ id: 1 }), account({ id: 2, hidden: true }), account({ id: 3 }), account({ id: 4, hidden: true })];
    const { shown, hidden } = splitHidden(all);
    expect(shown.map((a) => a.id)).toEqual([1, 3]);
    expect(hidden.map((a) => a.id)).toEqual([2, 4]);
  });

  it("has no hidden group when nothing is hidden", () => {
    expect(splitHidden([account()]).hidden).toEqual([]);
  });
});
