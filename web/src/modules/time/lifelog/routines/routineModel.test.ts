import type { RoutineOccurrence, RoutineStep } from "@/api/time";
import { MAX_LABEL_CHARS, MAX_NAME_CHARS, MAX_STEPS_PER_ROUTINE, MAX_VALUE_CHARS } from "./routineLimits";
import { emptyStep, incompleteDays, isDraftReady, matchChoices, moveStep, stepRows, stepsOfSuggestion, suggestedName, withCurrent } from "./routineModel";

const span = (seconds: number) => ({ start: "2026-05-04T08:00:00Z", end: "2026-05-04T09:00:00Z", seconds });
const day = (date: string, steps: (ReturnType<typeof span> | null)[]): RoutineOccurrence => ({
  date,
  complete: steps.every((s) => s !== null),
  steps,
  gaps: [],
  door_to_door_seconds: null,
});

describe("routine model", () => {
  it("names a long suggestion by its first and last description and a short one by all of them", () => {
    expect(suggestedName({ steps: ["a", "b", "c"], days: 6 })).toBe("a → c");
    expect(suggestedName({ steps: ["a", "b"], days: 6 })).toBe("a → b");
    expect(suggestedName({ steps: ["x".repeat(60), "y", "z".repeat(60)], days: 6 }).length).toBe(80);
  });

  it("turns a suggestion into description Steps labelled with the description", () => {
    expect(stepsOfSuggestion({ steps: ["a", "b"], days: 5 })).toEqual([
      { label: "a", kind: "description", value: "a" },
      { label: "b", kind: "description", value: "b" },
    ]);
  });

  it("cuts a long description to a label that fits, keeping the whole description as the match", () => {
    const long = "word ".repeat(30).trim();
    const [step] = stepsOfSuggestion({ steps: [long, "b"], days: 5 });
    expect(step?.value).toBe(long);
    expect(Array.from(step?.label ?? "").length).toBe(MAX_LABEL_CHARS);
    expect(step?.label.endsWith("…")).toBe(true);
    expect(isDraftReady("n", stepsOfSuggestion({ steps: [long, "b"], days: 5 }))).toBe(true);
  });

  it("moves a Step one place and stops at the ends", () => {
    expect(moveStep(["a", "b", "c"], 2, -1)).toEqual(["a", "c", "b"]);
    expect(moveStep(["a", "b", "c"], 0, -1)).toEqual(["a", "b", "c"]);
    expect(moveStep(["a", "b", "c"], 2, 1)).toEqual(["a", "b", "c"]);
  });

  it("is ready only with a name and one to eight complete Steps", () => {
    const ok: RoutineStep = { label: "a", kind: "description", value: "b" };
    expect(isDraftReady("Office", [ok])).toBe(true);
    expect(isDraftReady(" ", [ok])).toBe(false);
    expect(isDraftReady("n", [])).toBe(false);
    expect(isDraftReady("n", [emptyStep()])).toBe(false);
    expect(isDraftReady("n", Array.from({ length: 9 }, () => ok))).toBe(false);
  });

  it("refuses what the server would refuse: a name, label or match that is too long, counted in characters", () => {
    const ok: RoutineStep = { label: "a", kind: "description", value: "b" };
    expect(isDraftReady("n".repeat(MAX_NAME_CHARS), [ok])).toBe(true);
    expect(isDraftReady("n".repeat(MAX_NAME_CHARS + 1), [ok])).toBe(false);
    expect(isDraftReady("n", [{ ...ok, label: "l".repeat(MAX_LABEL_CHARS + 1) }])).toBe(false);
    expect(isDraftReady("n", [{ ...ok, value: "v".repeat(MAX_VALUE_CHARS + 1) }])).toBe(false);
    // Emoji are one character to the server but two to a string's length.
    expect(isDraftReady("😀".repeat(MAX_NAME_CHARS), [ok])).toBe(true);
    expect(isDraftReady("n", Array.from({ length: MAX_STEPS_PER_ROUTINE }, () => ok))).toBe(true);
  });

  it("offers the Period's descriptions and projects, sorted", () => {
    const totals = {
      logged_seconds: 0,
      by_project: [{ name: "office", color: null, seconds: 1, entries: 1, days: 1, descriptions: [] }],
      by_description: [
        { description: "walk", seconds: 1, entries: 1, days: 1 },
        { description: "bath", seconds: 1, entries: 1, days: 1 },
      ],
    };
    expect(matchChoices(totals)).toEqual({ description: ["bath", "walk"], project: ["office"] });
    expect(matchChoices(undefined)).toEqual({ description: [], project: [] });
  });

  it("keeps a saved Step's own value among the options", () => {
    expect(withCurrent(["a", "b"], "old")).toEqual(["old", "a", "b"]);
    expect(withCurrent(["a", "b"], "b")).toEqual(["a", "b"]);
    expect(withCurrent(["a"], " ")).toEqual(["a"]);
  });

  it("makes a row per Day with hours per Step, zero for a missing Step", () => {
    const rows = stepRows([day("2026-05-04", [span(3600), null, span(5400)])]);
    expect(rows[0]).toMatchObject({ date: "2026-05-04", complete: false, s0: 1, s1: 0, s2: 1.5, seconds: [3600, null, 5400] });
  });

  it("lists incomplete Days with the labels of the Steps they lack", () => {
    const steps: RoutineStep[] = [
      { label: "To office", kind: "description", value: "a" },
      { label: "At office", kind: "description", value: "b" },
    ];
    const found = incompleteDays([day("2026-05-04", [span(1), span(1)]), day("2026-05-05", [null, span(1)])], steps);
    expect(found).toEqual([{ date: "2026-05-05", missing: ["To office"] }]);
  });
});
