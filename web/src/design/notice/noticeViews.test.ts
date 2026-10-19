import { Info } from "lucide-react";
import { NOTICE_VIEWS, viewFor, WARNING_STYLE } from "./noticeViews";

describe("viewFor", () => {
  it("finds a built-in kind", () => {
    expect(viewFor("failure")).toBe(NOTICE_VIEWS.failure);
  });

  it("finds a kind a Module added", () => {
    const view = { icon: Info, ...WARNING_STYLE };
    expect(viewFor("settings-fallback", { "settings-fallback": view })).toBe(view);
  });

  it("falls back to the built-in kind when the Module added others", () => {
    expect(viewFor<"other">("empty", { other: NOTICE_VIEWS.failure })).toBe(NOTICE_VIEWS.empty);
  });

  it("gives warning and an added warning-style kind the same tone", () => {
    expect(NOTICE_VIEWS.warning.tone).toBe(WARNING_STYLE.tone);
  });
});
