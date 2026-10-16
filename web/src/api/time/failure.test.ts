import { RequestError } from "../callApi";
import { failureFromCode, toTimeFailure } from "./failure";

const problem = (status: number, body: Record<string, unknown>) =>
  new RequestError("detail", status >= 500 ? "unreachable" : "rejected", status, body);

describe("toTimeFailure", () => {
  it("recognises an invalid token", () => {
    expect(toTimeFailure(problem(401, { code: "invalid_token" }))).toEqual({ kind: "invalid_token" });
  });

  it("carries the reset time of a rate limit", () => {
    expect(toTimeFailure(problem(429, { code: "rate_limited", resets_in_secs: 90 }))).toEqual({
      kind: "rate_limited",
      resetsInSecs: 90,
    });
  });

  it("copes with a rate limit that has no reset time", () => {
    expect(toTimeFailure(problem(429, { code: "rate_limited" }))).toEqual({ kind: "rate_limited" });
  });

  it("tells Toggl being unavailable from the server being unreachable", () => {
    expect(toTimeFailure(problem(502, { code: "unreachable" }))).toEqual({ kind: "toggl_unavailable" });
    expect(toTimeFailure(new RequestError("down", "unreachable"))).toEqual({ kind: "server_unreachable" });
  });

  it("falls back to the server's message for anything else", () => {
    expect(toTimeFailure(new RequestError("Toggl is not connected.", "rejected", 409))).toEqual({
      kind: "other",
      message: "Toggl is not connected.",
    });
    expect(toTimeFailure("boom")).toEqual({ kind: "other", message: "Something went wrong." });
  });

  it("maps a stored run's error code to the same failures", () => {
    expect(failureFromCode("invalid_token")).toEqual({ kind: "invalid_token" });
    expect(failureFromCode("malformed")).toEqual({ kind: "toggl_unavailable" });
    expect(failureFromCode("no_snapshot")).toEqual({ kind: "no_snapshot" });
    expect(failureFromCode("snapshot_unreadable")).toEqual({ kind: "snapshot_unreadable" });
    expect(failureFromCode("sync_in_progress")).toEqual({ kind: "sync_in_progress" });
    expect(failureFromCode("connection_changed")).toEqual({ kind: "connection_changed" });
    expect(failureFromCode("database")).toBeNull();
    expect(failureFromCode(null)).toBeNull();
  });
});
