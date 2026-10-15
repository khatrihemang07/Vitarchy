import createClient from "openapi-fetch";
import type { paths } from "./schema";

/** Empty means same origin; Tauri builds point this at the local server. */
const apiBaseUrl: string = import.meta.env.VITE_API_BASE_URL ?? "";

export const api = createClient<paths>({
  // Request needs an absolute URL; same-origin is the default.
  baseUrl: apiBaseUrl || globalThis.location.origin,
  // Resolved per call so tests can stub the global fetch.
  fetch: (request) => globalThis.fetch(request),
});
