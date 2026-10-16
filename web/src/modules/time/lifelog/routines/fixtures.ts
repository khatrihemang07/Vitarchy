/** Test helper: routes and bodies of the Routines API. */
export const SUGGESTIONS = "GET /api/time/routines/suggestions";
export const ROUTINES = "GET /api/time/routines";
export const CREATE = "POST /api/time/routines";

export const period = { from: "2026-05-01", to: "2026-05-31", tz: "UTC" };

export const OFFICE_SUGGESTION = { steps: ["travel to_office", "inside_office", "travel from_office"], days: 8 };

export const savedRoutine = {
  id: 3,
  name: "Office day",
  steps: [
    { label: "To office", kind: "description", value: "travel to_office" },
    { label: "At office", kind: "description", value: "inside_office" },
  ],
  created_at: "2026-05-01T00:00:00Z",
  updated_at: "2026-05-01T00:00:00Z",
};

const span = (seconds: number) => ({ start: "2026-05-04T08:00:00Z", end: "2026-05-04T09:00:00Z", seconds });
const spread = { median_seconds: 3600, p25_seconds: 3300, p75_seconds: 3900 };
const clock = { median_minute: 480, p25_minute: 470, p75_minute: 490 };

/** Two Days of the saved Routine: one complete, one without its first Step. */
export const occurrencesBody = {
  routine: savedRoutine,
  occurrences: [
    { date: "2026-05-04", complete: true, steps: [span(3600), span(28800)], gaps: [900], door_to_door_seconds: 33300 },
    { date: "2026-05-05", complete: false, steps: [null, span(28000)], gaps: [null], door_to_door_seconds: null },
  ],
  stats: {
    complete_days: 1,
    incomplete_days: 1,
    door_to_door: { median_seconds: 33300, p25_seconds: 33300, p75_seconds: 33300 },
    steps: [
      { days: 1, start: clock, end: clock, duration: spread },
      { days: 2, start: clock, end: clock, duration: { ...spread, median_seconds: 28400 } },
    ],
    gaps: [spread],
    weekdays: [],
  },
};
