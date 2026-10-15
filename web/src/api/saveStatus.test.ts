import { initialSaveStatus, type SaveEvent, saveStateOf, saveStatusReducer } from "./saveStatus";

const run = (...events: SaveEvent[]) => events.reduce(saveStatusReducer, initialSaveStatus);
const started = (id: number): SaveEvent => ({ type: "started", id });
const succeeded = (id: number): SaveEvent => ({ type: "succeeded", id });
const failed = (id: number, message = "nope"): SaveEvent => ({ type: "failed", id, message });

describe("saveStateOf", () => {
  it("is idle before any save", () => {
    expect(saveStateOf(run())).toEqual({ kind: "idle" });
  });

  it("is saving while any save is in flight", () => {
    expect(saveStateOf(run(started(1), started(2), succeeded(1)))).toEqual({ kind: "saving" });
  });

  it("is saved once every save has succeeded", () => {
    expect(saveStateOf(run(started(1), succeeded(1)))).toEqual({ kind: "saved" });
  });

  it("shows the error of a failed save", () => {
    expect(saveStateOf(run(started(1), failed(1, "too long")))).toEqual({
      kind: "error",
      message: "too long",
    });
  });

  it("keeps the error when a save started before the failure then succeeds", () => {
    const status = run(started(1), started(2), failed(1), succeeded(2));
    expect(saveStateOf(status).kind).toBe("error");
  });

  it("clears the error when a save started after the failure succeeds", () => {
    const status = run(started(1), failed(1), started(2), succeeded(2));
    expect(saveStateOf(status)).toEqual({ kind: "saved" });
  });

  it("keeps showing the error while a retry is still saving", () => {
    expect(saveStateOf(run(started(1), failed(1), started(2))).kind).toBe("error");
  });

  it("reports the latest failure", () => {
    const status = run(started(1), failed(1, "a"), started(2), failed(2, "b"));
    expect(saveStateOf(status)).toEqual({ kind: "error", message: "b" });
  });
});
