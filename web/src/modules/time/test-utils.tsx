import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render } from "@testing-library/react";
import type { ReactElement } from "react";
import type { Source, SourceActivityStatus, TimeStatus } from "@/api/time";

/** A `Response` carrying JSON (problem documents use the problem content type). */
export function jsonResponse(status: number, body: unknown): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { "content-type": status >= 400 ? "application/problem+json" : "application/json" },
  });
}

/** One Source's figures for a person who has imported nothing and has no Snapshot. */
export function noActivity(source: Source): SourceActivityStatus {
  return { source, activity_count: 0, app_count: 0 };
}

/** Every Source with nothing imported, in the server's order. */
export const noActivities: SourceActivityStatus[] = [noActivity("toggl"), noActivity("clockify")];

/** A Timeline with no Segments yet. */
export const emptyTimeline: TimeStatus["timeline"] = { segment_count: 0, covered_secs: 0, away_secs: 0, unknown_secs: 0 };

/** Status of a person who has not connected Toggl. */
export const disconnected: TimeStatus = {
  connected: false,
  time_entry_count: 0,
  activity: noActivities,
  timeline: emptyTimeline,
};

/** Status of a connected person who has synced once. */
export const synced: TimeStatus = {
  connected: true,
  workspace_name: "My Workspace",
  time_entry_count: 2,
  activity: noActivities,
  timeline: emptyTimeline,
  oldest_entry: "2026-04-01T10:00:00Z",
  newest_entry: "2026-04-05T10:00:00Z",
  last_sync: {
    kind: "sync",
    started_at: "2026-04-10T12:00:00Z",
    finished_at: "2026-04-10T12:00:02Z",
    outcome: "success",
    counts: { inserted: 2 } as never,
    error_code: null,
  },
};

type Handler = (request: Request) => Response | Promise<Response>;

/**
 * Stubs `fetch` with handlers keyed `"METHOD /path"`. A handler may be a list:
 * each call takes the next entry, the last one repeating.
 */
export function stubApi(routes: Record<string, Handler | Handler[]>) {
  const counters = new Map<string, number>();
  const fetchMock = vi.fn((request: Request) => {
    const key = `${request.method} ${new URL(request.url).pathname}`;
    const entry = routes[key];
    if (!entry) return Promise.resolve(jsonResponse(404, { detail: `no stub for ${key}` }));
    const handlers = Array.isArray(entry) ? entry : [entry];
    const index = counters.get(key) ?? 0;
    counters.set(key, index + 1);
    const handler = handlers.at(Math.min(index, handlers.length - 1));
    return Promise.resolve(handler ? handler(request) : jsonResponse(404, { detail: "no handler" }));
  });
  vi.stubGlobal("fetch", fetchMock);
  return fetchMock;
}

/** Renders `ui` with a fresh query cache. */
export function renderWithClient(ui: ReactElement) {
  return render(
    <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}>
      {ui}
    </QueryClientProvider>,
  );
}
