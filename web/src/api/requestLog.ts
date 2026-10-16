import type { Middleware } from "openapi-fetch";
import { createLogger, type Logger } from "@/lib/log";

/** The id the server stamps on every response; quote it to find the matching server log line. */
const REQUEST_ID_HEADER = "x-request-id";

/** The frontend polls this every few seconds; its failures are expected while the server is down. */
const HEALTH_PATH = "/api/health";

/**
 * The one rule for how loudly a failed call is logged: a health poll is always
 * `debug` (a down server would otherwise log every few seconds); elsewhere a
 * 4xx is the caller's fault (`warn`) and a 5xx or network failure is `error`.
 */
export function failureLevel(path: string, outcome: number | "network"): "debug" | "warn" | "error" {
  if (path === HEALTH_PATH) return "debug";
  return typeof outcome === "number" && outcome < 500 ? "warn" : "error";
}

/** Path only: query strings can carry values we must not log. */
function describeRequest(request: Request) {
  return { method: request.method, path: new URL(request.url).pathname };
}

/**
 * Logs failed API calls once, at the boundary, so call sites need not.
 * Levels come from [`failureLevel`]. Bodies are never logged.
 */
export function requestLogMiddleware(log: Logger = createLogger("api")): Middleware {
  return {
    onResponse({ request, response }) {
      if (response.status < 400) return;
      const requestId = response.headers.get(REQUEST_ID_HEADER);
      const described = describeRequest(request);
      const fields = {
        ...described,
        status: response.status,
        ...(requestId && { requestId }),
      };
      log[failureLevel(described.path, response.status)]("request failed", fields);
    },
    onError({ request, error }) {
      const message = error instanceof Error ? error.message : String(error);
      const described = describeRequest(request);
      log[failureLevel(described.path, "network")]("network error", { ...described, error: message });
    },
  };
}
