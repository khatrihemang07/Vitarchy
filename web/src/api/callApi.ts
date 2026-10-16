/** Why a request failed: the server refused it, or it could not be reached. */
export type RequestErrorKind = "rejected" | "unreachable";

/**
 * A failed request carrying a message fit to show the user. The message may
 * quote what the person entered, so log `kind` and `status` instead of it.
 */
export class RequestError extends Error {
  constructor(
    message: string,
    readonly kind: RequestErrorKind,
    /** HTTP status, when the server answered at all. */
    readonly status?: number,
  ) {
    super(message);
  }
}

const UNREACHABLE = "Could not reach the server.";

function problemDetail(error: unknown): string {
  if (typeof error === "object" && error !== null && "detail" in error) {
    const { detail } = error;
    if (typeof detail === "string" && detail) return detail;
  }
  return "The server rejected this change.";
}

/** Runs a request, mapping a network failure or a problem response to a RequestError. */
export async function callApi<T>(
  request: () => Promise<{ data?: T; error?: unknown; response: { status: number } }>,
): Promise<T> {
  let result;
  try {
    result = await request();
  } catch {
    throw new RequestError(UNREACHABLE, "unreachable");
  }
  if (result.data === undefined) {
    // A proxy answers 5xx (not a problem document) when the server is down.
    const { status } = result.response;
    if (status >= 500) throw new RequestError(UNREACHABLE, "unreachable", status);
    throw new RequestError(problemDetail(result.error), "rejected", status);
  }
  return result.data;
}
