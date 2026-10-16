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
    /** The problem document, when the server sent one; read it for extra fields such as `code`. */
    readonly problem?: Record<string, unknown>,
  ) {
    super(message);
  }
}

const UNREACHABLE = "Could not reach the server.";

function asProblem(error: unknown): Record<string, unknown> | undefined {
  return typeof error === "object" && error !== null ? (error as Record<string, unknown>) : undefined;
}

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
  const { status } = result.response;
  // A success with no body (204) has neither data nor error.
  const emptySuccess = result.error === undefined && status < 400;
  if (result.data === undefined && !emptySuccess) {
    const problem = asProblem(result.error);
    // A proxy answers 5xx (not a problem document) when the server is down.
    if (status >= 500) throw new RequestError(UNREACHABLE, "unreachable", status, problem);
    throw new RequestError(problemDetail(result.error), "rejected", status, problem);
  }
  return result.data as T;
}
