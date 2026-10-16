# Analysis is computed on the server

Time analysis (totals, splits at Day boundaries, Routine matching and Insights) is done in pure Rust functions over rows that SQL fetches, in the Time crate. The web app receives ready-to-draw series and only renders them. We chose this over shipping raw rows to the browser because Activity alone is tens of thousands of rows and growing, the Day boundary logic must be exact and tested once, and Rust tests against stage are the project's strongest safety net. Recharts is the only chart dependency, bundled locally and themed by design tokens, so the app still loads nothing from the internet (ADR 0001).

## Considered Options

- **Aggregate in the browser.** Rejected: large payloads, and time zone and DST logic would need testing twice.
- **Aggregate in SQL only.** Rejected: Day splitting across DST and Insights are awkward to express and test in SQL.
- **A heavier chart stack (D3 everywhere, or a hosted service).** Rejected: more code to own, or a network dependency. Heatmaps and timelines stay small custom SVG.

## Consequences

- The browser sends its IANA time zone as `tz` on every analysis request, because a Day is local.
- A new analysis is a pure function with unit tests, a thin route, and a chart component.
