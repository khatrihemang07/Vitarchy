/**
 * Vitest global setup: pins the time zone so date logic is tested against a zone that
 * is neither UTC nor the developer's own. Los Angeles has daylight saving, so 1 Nov
 * 2026 is a 25-hour Day. Tests build their instants as explicit UTC strings.
 */
export function setup(): void {
  process.env.TZ = "America/Los_Angeles";
}
