import { PERIOD_TABS, TIME_TABS } from "./tabs";

describe("PERIOD_TABS", () => {
  it("is every tab that explains an empty Period, in tab order", () => {
    expect(PERIOD_TABS).toEqual(TIME_TABS.filter((tab) => "empty" in tab).map((tab) => tab.id));
    expect(PERIOD_TABS).not.toContain("sources");
  });
});
