import createClient from "openapi-fetch";
import type { Logger } from "@/lib/log";
import { failureLevel, requestLogMiddleware } from "./requestLog";

function spyLogger() {
  return { debug: vi.fn(), info: vi.fn(), warn: vi.fn(), error: vi.fn() } satisfies Logger;
}

/** A client wired like the real one, over a stubbed fetch. */
function clientWith(respond: () => Promise<Response>) {
  const log = spyLogger();
  const client = createClient({
    baseUrl: "http://localhost",
    fetch: () => respond(),
  });
  client.use(requestLogMiddleware(log));
  return { client, log };
}

const reply = (status: number, headers: Record<string, string> = {}) =>
  Promise.resolve(new Response("{}", { status, headers }));

describe("requestLogMiddleware", () => {
  it("warns on a 4xx with method, path, status and request id", async () => {
    const { client, log } = clientWith(() => reply(422, { "x-request-id": "abc-123" }));
    await client.PUT("/api/settings/core" as never, { body: {} } as never);
    expect(log.warn).toHaveBeenCalledWith("request failed", {
      method: "PUT",
      path: "/api/settings/core",
      status: 422,
      requestId: "abc-123",
    });
    expect(log.error).not.toHaveBeenCalled();
  });

  it("errors on a 5xx and omits a missing request id", async () => {
    const { client, log } = clientWith(() => reply(503));
    await client.GET("/api/settings" as never);
    expect(log.error).toHaveBeenCalledWith("request failed", {
      method: "GET",
      path: "/api/settings",
      status: 503,
    });
  });

  it("stays quiet on success", async () => {
    const { client, log } = clientWith(() => reply(200));
    await client.GET("/api/settings" as never);
    expect(log.warn).not.toHaveBeenCalled();
    expect(log.error).not.toHaveBeenCalled();
  });

  it("logs a network failure as an error and lets it propagate", async () => {
    const { client, log } = clientWith(() => Promise.reject(new TypeError("Failed to fetch")));
    await expect(client.GET("/api/settings" as never)).rejects.toThrow("Failed to fetch");
    expect(log.error).toHaveBeenCalledWith("network error", {
      method: "GET",
      path: "/api/settings",
      error: "Failed to fetch",
    });
  });

  it("never logs the query string", async () => {
    const { client, log } = clientWith(() => reply(400));
    await client.GET("/api/x" as never, { params: { query: { token: "s3cret" } } } as never);
    expect(JSON.stringify(log.warn.mock.calls)).not.toContain("s3cret");
  });

  it("demotes a failing health poll to debug so a down server does not spam errors", async () => {
    const down = clientWith(() => reply(503));
    await down.client.GET("/api/health" as never);
    expect(down.log.debug).toHaveBeenCalledWith("request failed", {
      method: "GET",
      path: "/api/health",
      status: 503,
    });
    expect(down.log.error).not.toHaveBeenCalled();

    const offline = clientWith(() => Promise.reject(new TypeError("Failed to fetch")));
    await expect(offline.client.GET("/api/health" as never)).rejects.toThrow();
    expect(offline.log.debug).toHaveBeenCalledWith("network error", expect.anything());
    expect(offline.log.error).not.toHaveBeenCalled();
  });

  it("does not log request bodies, so entered values stay out of failure logs", async () => {
    const { client, log } = clientWith(() => reply(400));
    await client.PUT("/api/settings/core" as never, { body: { display_name: "SECRETNAME" } } as never);
    expect(JSON.stringify([log.warn.mock.calls, log.error.mock.calls])).not.toContain("SECRETNAME");
    expect(log.warn).toHaveBeenCalled();
  });
});

describe("failureLevel", () => {
  it("is the one rule: health is always debug, otherwise 4xx warn and 5xx or network error", () => {
    expect(failureLevel("/api/health", 503)).toBe("debug");
    expect(failureLevel("/api/health", "network")).toBe("debug");
    expect(failureLevel("/api/settings", 404)).toBe("warn");
    expect(failureLevel("/api/settings", 500)).toBe("error");
    expect(failureLevel("/api/settings", "network")).toBe("error");
  });
});
