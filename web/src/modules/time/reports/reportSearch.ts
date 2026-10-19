import type { Grouping } from "@/api/time";
import { COMPARE_MODES, type CompareMode, defaultCompare } from "./compareMode";
import { parsePeriodSearch, type PeriodSearch, periodParams } from "@/lib/period/periodSearch";

/** Whether group stats show daily totals or single entries. */
export type StatMode = "per_day" | "per_entry";

/** The choices that narrow a Report and are remembered between visits. */
export interface ReportFilters {
  project: number[];
  tag: string[];
  client: number[];
}

/** Everything the URL of the Reports page says. */
export interface ReportSearch extends ReportFilters, PeriodSearch {
  /** Description search, ignoring case. */
  q: string;
  group1: Grouping;
  group2?: Grouping;
  mode: StatMode;
  /** The comparison the person chose; absent means the default for the kind. */
  compare?: CompareMode;
}

const GROUPINGS: readonly Grouping[] = ["description", "project", "tag"];
const MODES: readonly StatMode[] = ["per_day", "per_entry"];
const MAX_QUERY_LENGTH = 200;

const pick = <T extends string>(options: readonly T[], value: unknown): T | undefined =>
  options.find((option) => option === value);

const asList = (value: unknown): unknown[] => (Array.isArray(value) ? value : value === undefined ? [] : [value]);

function ids(value: unknown): number[] {
  const found = asList(value).filter((v): v is number => typeof v === "number" && Number.isInteger(v) && v > 0);
  return [...new Set(found)];
}

function names(value: unknown): string[] {
  const found = asList(value)
    .filter((v) => typeof v === "string" || typeof v === "number")
    .map(String)
    .filter((v) => v !== "");
  return [...new Set(found)];
}

/** Time's Periods default to a week. */
export const parseTimePeriod = (raw: Record<string, unknown>): PeriodSearch => parsePeriodSearch(raw, "week");

/** The URL form of a Period in Time: a week is the default and left out. */
export const timePeriodParams = (search: PeriodSearch): Record<string, unknown> => periodParams(search, "week");

/** Turns whatever the URL held into a valid state; the URL is user-editable, so nothing is trusted. */
export function parseReportSearch(raw: Record<string, unknown>): ReportSearch {
  const group1 = pick(GROUPINGS, raw.group1) ?? "description";
  const group2 = pick(GROUPINGS, raw.group2);
  const q = typeof raw.q === "string" || typeof raw.q === "number" ? String(raw.q).slice(0, MAX_QUERY_LENGTH) : "";
  const compare = pick(COMPARE_MODES, raw.compare);
  return {
    ...parseTimePeriod(raw),
    project: ids(raw.project),
    tag: names(raw.tag),
    client: ids(raw.client),
    q,
    group1,
    ...(group2 && group2 !== group1 && { group2 }),
    mode: pick(MODES, raw.mode) ?? "per_day",
    ...(compare && { compare }),
  };
}

/** The URL form of a state: defaults are left out. */
export function toSearchParams(search: ReportSearch): Record<string, unknown> {
  return {
    ...timePeriodParams(search),
    ...(search.project.length > 0 && { project: search.project }),
    ...(search.tag.length > 0 && { tag: search.tag }),
    ...(search.client.length > 0 && { client: search.client }),
    ...(search.q && { q: search.q }),
    ...(search.group1 !== "description" && { group1: search.group1 }),
    ...(search.group2 && { group2: search.group2 }),
    ...(search.mode !== "per_day" && { mode: search.mode }),
    ...(search.compare && search.compare !== defaultCompare(search.kind) && { compare: search.compare }),
  };
}

/** True when the URL already chooses a project, tag or client (then remembered choices are not applied). */
export function hasFilters(search: ReportFilters): boolean {
  return search.project.length + search.tag.length + search.client.length > 0;
}
