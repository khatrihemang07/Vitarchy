import type { components } from "../schema";

/** What `GET /api/time/status` returns. */
export type TimeStatus = components["schemas"]["TimeStatus"];
/** One Sync or Import attempt. */
export type TimeRun = components["schemas"]["RunInfo"];
/** What a Sync did. */
export type SyncCounts = components["schemas"]["SyncCounts"];

/** What an Import did. */
export type ImportCounts = components["schemas"]["ImportCounts"];

/** Imported Activity, the Snapshot and the latest Import. */
export type ActivityStatus = components["schemas"]["ActivityStatus"];

/** Screen time by app, Context and title. */
export type ActivityTotals = components["schemas"]["Totals"];
/** One app's time. */
export type ActivityAppTotal = components["schemas"]["AppTotal"];
/** One Context's time within an app. */
export type ActivityContextTotal = components["schemas"]["ContextTotal"];
/** Seconds by weekday (Monday first) and hour. */
export type ActivityHeatmap = components["schemas"]["Heatmap"];
/** First and last Activity of one Day. */
export type ActivityDayBound = components["schemas"]["DayBound"];
/** One Day's lane of Activity spans. */
export type ActivityTimeline = components["schemas"]["DayTimeline"];
/** One stretch of Activity in one Context. */
export type ActivitySpan = components["schemas"]["Span"];
/** Weekly screen time for the busiest apps. */
export type ActivityTrend = components["schemas"]["Trend"];
/** Everything the Activity tab draws for a Period, from one response. */
export type ActivityOverview = components["schemas"]["Overview"];
/** Which weekday a week starts on. */
export type WeekStart = components["schemas"]["WeekStart"];

/** Everything the Life log tab draws for a Period, from one response. */
export type LifeLogOverview = components["schemas"]["LifeLogOverview"];
/** One Day's hours per project and its unlogged time. */
export type LifeLogDaySplit = components["schemas"]["DaySplit"];
/** Logged time by project and by description. */
export type LifeLogTotals = components["schemas"]["LifeLogTotals"];
/** One project's time, with its descriptions. */
export type LifeLogProjectTotal = components["schemas"]["ProjectTotal"];
/** One description's time. */
export type LifeLogDescriptionTotal = components["schemas"]["DescriptionTotal"];
/** When a frequent description usually starts, ends and how long it lasts. */
export type LifeLogDescriptionStat = components["schemas"]["DescriptionStat"];
/** One Day's lane of Time Entries. */
export type LifeLogTimeline = components["schemas"]["LifeLogTimeline"];
/** One Time Entry's part of a Day. */
export type LifeLogSpan = components["schemas"]["LifeLogSpan"];

/** A saved Routine: ordered Steps matched to Time Entries. */
export type Routine = components["schemas"]["Routine"];
/** One Step of a Routine, as saved. */
export type RoutineStep = components["schemas"]["Step"];
/** Whether a Step matches a description or a project. */
export type RoutineMatchKind = components["schemas"]["MatchKind"];
/** What `POST` and `PUT` take: a name and Steps. */
export type RoutineDraft = components["schemas"]["RoutineRequest"];
/** A recurring sequence of descriptions offered as a Routine. */
export type RoutineSuggestion = components["schemas"]["Suggestion"];
/** One Day on which a Routine happened. */
export type RoutineOccurrence = components["schemas"]["Occurrence"];
/** When one Step happened on a Day. */
export type RoutineStepSpan = components["schemas"]["StepSpan"];
/** A Routine's Occurrences over a Period, with statistics. */
export type RoutineOccurrences = components["schemas"]["RoutineOccurrences"];
/** The usual times and lengths of a Routine. */
export type RoutineStats = components["schemas"]["RoutineStats"];
