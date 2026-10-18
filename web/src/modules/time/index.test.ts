import { timeModule } from "./index";

describe("timeModule", () => {
  it("is mounted at /time under the time id with the Time accent", () => {
    expect(timeModule.id).toBe("time");
    expect(timeModule.navItem.to).toBe("/time");
    expect(timeModule.accent).toBe("var(--module-time)");
  });

  it("lists Sources, Reports and Timeline as pages for the command bar", () => {
    expect(timeModule.pages?.map((page) => page.to)).toEqual(["/time", "/time/reports", "/time/timeline"]);
  });
});
