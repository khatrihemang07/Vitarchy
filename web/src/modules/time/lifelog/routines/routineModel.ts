/** Shaping Routines for the screen: names, Steps, chart rows. Pure. */
import type { LifeLogTotals, RoutineOccurrence, RoutineStep, RoutineSuggestion } from "@/api/time";
import { toHours } from "../../shared/duration";

/** The most Steps a Routine may have (the server enforces it too). */
export const MAX_STEPS = 8;
/** The longest Routine name (the server enforces it too). */
export const MAX_NAME_CHARS = 80;

const ARROW = " → ";

/** A name to start from: the suggestion's first and last descriptions, cut to the name limit. */
export function suggestedName(suggestion: RoutineSuggestion): string {
  const first = suggestion.steps[0] ?? "";
  const last = suggestion.steps.at(-1) ?? "";
  const name = suggestion.steps.length > 2 ? `${first}${ARROW}${last}` : suggestion.steps.join(ARROW);
  return name.length > MAX_NAME_CHARS ? `${name.slice(0, MAX_NAME_CHARS - 1)}…` : name;
}

/** The Steps a confirmed suggestion becomes: each description is its own label. */
export function stepsOfSuggestion(suggestion: RoutineSuggestion): RoutineStep[] {
  return suggestion.steps.map((description) => ({ label: description, kind: "description", value: description }));
}

/** Identity of a suggestion, for hiding it. */
export function suggestionKey(suggestion: RoutineSuggestion): string {
  return suggestion.steps.join("\u0001");
}

/** A new, empty Step. */
export function emptyStep(): RoutineStep {
  return { label: "", kind: "description", value: "" };
}

/** `steps` with the one at `index` moved by `delta` places; unchanged when that would leave the list. */
export function moveStep<T>(steps: readonly T[], index: number, delta: -1 | 1): T[] {
  const target = index + delta;
  const moved = steps[index];
  if (moved === undefined || target < 0 || target >= steps.length) return [...steps];
  const next = [...steps];
  next.splice(index, 1);
  next.splice(target, 0, moved);
  return next;
}

/** Whether a name and Steps are worth sending: the same rules the server checks, so the button can wait. */
export function isDraftReady(name: string, steps: readonly RoutineStep[]): boolean {
  return (
    name.trim() !== "" &&
    name.trim().length <= MAX_NAME_CHARS &&
    steps.length >= 1 &&
    steps.length <= MAX_STEPS &&
    steps.every((step) => step.label.trim() !== "" && step.value.trim() !== "")
  );
}

/** What a Step can be matched to, from what the Period's Time Entries hold. */
export interface MatchChoices {
  description: string[];
  project: string[];
}

/** The descriptions and projects the Period's Time Entries hold, sorted. Empty until the totals load. */
export function matchChoices(totals: LifeLogTotals | undefined): MatchChoices {
  const sorted = (names: readonly string[]) => [...new Set(names)].sort((a, b) => a.localeCompare(b));
  return {
    description: sorted(totals?.by_description.map((d) => d.description) ?? []),
    project: sorted(totals?.by_project.map((p) => p.name) ?? []),
  };
}

/** `options` with `current` among them, so a saved Step keeps its own value even when the Period lacks it. */
export function withCurrent(options: readonly string[], current: string): string[] {
  return current.trim() === "" || options.includes(current) ? [...options] : [current, ...options];
}

/** One Day's bar: hours per Step (0 for a missing Step) and the exact seconds. */
export interface StepRow {
  date: string;
  complete: boolean;
  /** Seconds per Step; `null` when the Step did not happen. */
  seconds: (number | null)[];
  [key: string]: unknown;
}

/** The key of the `index`th Step in a chart row. */
export const stepKey = (index: number) => `s${String(index)}`;

/** A row per Day for the stacked bars. */
export function stepRows(occurrences: readonly RoutineOccurrence[]): StepRow[] {
  return occurrences.map((day) => {
    const seconds = day.steps.map((span) => span?.seconds ?? null);
    const row: StepRow = { date: day.date, complete: day.complete, seconds };
    seconds.forEach((value, index) => {
      row[stepKey(index)] = toHours(value ?? 0);
    });
    return row;
  });
}

/** The Days with a Step missing, and which Steps (by label). */
export function incompleteDays(occurrences: readonly RoutineOccurrence[], steps: readonly RoutineStep[]): { date: string; missing: string[] }[] {
  return occurrences
    .filter((day) => !day.complete)
    .map((day) => ({
      date: day.date,
      missing: day.steps.flatMap((span, index) => (span === null ? [steps[index]?.label ?? `Step ${String(index + 1)}`] : [])),
    }));
}
