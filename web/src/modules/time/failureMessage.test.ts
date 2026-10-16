import { failureMessage } from "./failureMessage";

describe("failureMessage", () => {
  it("tells the person what to do about a rejected token", () => {
    expect(failureMessage({ kind: "invalid_token" })).toMatch(/rejected this API token/);
  });

  it("says when a rate limit resets", () => {
    expect(failureMessage({ kind: "rate_limited", resetsInSecs: 1800 })).toBe(
      "Toggl's request limit was reached. It resets in 30 min.",
    );
  });

  it("falls back to 'try again later' without a reset time", () => {
    expect(failureMessage({ kind: "rate_limited" })).toMatch(/Try again later/);
  });

  it("separates Toggl from the Vitarchy server", () => {
    expect(failureMessage({ kind: "toggl_unavailable" })).toMatch(/reach Toggl/);
    expect(failureMessage({ kind: "server_unreachable" })).toMatch(/Vitarchy server/);
  });

  it("asks the person to wait when a Sync is already running", () => {
    expect(failureMessage({ kind: "sync_in_progress" })).toMatch(/already running/);
  });

  it("asks for another Sync when the connection changed during one", () => {
    expect(failureMessage({ kind: "connection_changed" })).toMatch(/Sync again/);
  });

  it("passes through any other message", () => {
    expect(failureMessage({ kind: "other", message: "Toggl is not connected." })).toBe("Toggl is not connected.");
  });
});
