import { pendingTogether } from "./pendingTogether";

describe("pendingTogether", () => {
  it("is pending while any query is", () => {
    expect(pendingTogether([{ isPending: false }, { isPending: true }]).isPending).toBe(true);
  });

  it("is settled once every query has answered, failed or not", () => {
    expect(pendingTogether([{ isPending: false }, { isPending: false }]).isPending).toBe(false);
  });

  it("never reports a failure itself", () => {
    expect(pendingTogether([{ isPending: true }]).isError).toBe(false);
  });
});
