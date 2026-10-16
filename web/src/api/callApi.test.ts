import { callApi, RequestError } from "./callApi";

describe("callApi", () => {
  it("returns the data of a successful call", async () => {
    await expect(callApi(() => Promise.resolve({ data: 7, response: { status: 200 } }))).resolves.toBe(7);
  });

  it("shows the server's problem detail when it rejects the request", async () => {
    const rejected = callApi(() =>
      Promise.resolve({ error: { detail: "display_name too long" }, response: { status: 400 } }),
    );
    await expect(rejected).rejects.toThrow(new RequestError("display_name too long", "rejected", 400, { detail: "display_name too long" }));
  });

  it("falls back to a generic message when the problem has no detail", async () => {
    const rejected = callApi(() => Promise.resolve({ error: "nope", response: { status: 400 } }));
    await expect(rejected).rejects.toThrow("The server rejected this change.");
  });

  it("reports an unreachable server for a network failure or a proxy 5xx", async () => {
    await expect(callApi(() => Promise.reject(new Error("offline")))).rejects.toThrow(
      "Could not reach the server.",
    );
    await expect(callApi(() => Promise.resolve({ response: { status: 502 } }))).rejects.toThrow(
      "Could not reach the server.",
    );
  });

  it("tells a rejection (with its status) from an unreachable server", async () => {
    const rejected = await callApi(() =>
      Promise.resolve({ error: { detail: "x" }, response: { status: 409 } }),
    ).catch((e: unknown) => e);
    expect(rejected).toMatchObject({ kind: "rejected", status: 409 });

    const offline = await callApi(() => Promise.reject(new Error("offline"))).catch((e: unknown) => e);
    expect(offline).toMatchObject({ kind: "unreachable" });
    expect(offline).not.toHaveProperty("status", expect.anything());
  });

  it("accepts a success without a body, such as a 204", async () => {
    await expect(callApi(() => Promise.resolve({ response: { status: 204 } }))).resolves.toBeUndefined();
  });

  it("keeps the problem document so callers can read its extra fields", async () => {
    const failed = await callApi(() =>
      Promise.resolve({ error: { detail: "x", code: "rate_limited" }, response: { status: 429 } }),
    ).catch((e: unknown) => e);
    expect(failed).toMatchObject({ problem: { code: "rate_limited" } });
    const unreachable = await callApi(() =>
      Promise.resolve({ error: { code: "unreachable" }, response: { status: 502 } }),
    ).catch((e: unknown) => e);
    expect(unreachable).toMatchObject({ kind: "unreachable", problem: { code: "unreachable" } });
  });
});
