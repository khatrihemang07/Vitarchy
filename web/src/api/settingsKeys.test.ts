import { saveMutationKey, settingsKey } from "./settingsKeys";

describe("saveMutationKey", () => {
  it("is built from the settings key and names the scope", () => {
    expect(saveMutationKey("core")).toEqual([...settingsKey, "save", "core"]);
  });

  it("differs per scope so scopes do not count each other's pending saves", () => {
    expect(saveMutationKey("time")).not.toEqual(saveMutationKey("finance"));
  });
});
