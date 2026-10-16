import createClient from "openapi-fetch";
import { requestLogMiddleware } from "./requestLog";
import type { paths } from "./schema";

/** Empty means same origin; Tauri builds point this at the local server. */
const apiBaseUrl: string = import.meta.env.VITE_API_BASE_URL ?? "";

/** The typed API client; every server call goes through it so request logging applies. */
export const api = createClient<paths>({
  // Request needs an absolute URL; same-origin is the default.
  baseUrl: apiBaseUrl || globalThis.location.origin,
  // Resolved per call so tests can stub the global fetch.
  fetch: (request) => globalThis.fetch(request),
});

api.use(requestLogMiddleware());
