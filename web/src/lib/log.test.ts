import { createLogger, isEnabled, parseLogLevel } from "./log";

describe("parseLogLevel", () => {
  it("accepts every known level", () => {
    for (const level of ["debug", "info", "warn", "error", "silent"] as const) {
      expect(parseLogLevel(level, "warn")).toBe(level);
    }
  });

  it("falls back for missing or unknown values", () => {
    expect(parseLogLevel(undefined, "info")).toBe("info");
    expect(parseLogLevel("", "info")).toBe("info");
    expect(parseLogLevel("verbose", "error")).toBe("error");
  });
});

describe("isEnabled", () => {
  it("lets a message through at or above the threshold", () => {
    expect(isEnabled("warn", "warn")).toBe(true);
    expect(isEnabled("error", "warn")).toBe(true);
    expect(isEnabled("debug", "debug")).toBe(true);
  });

  it("drops a message below the threshold", () => {
    expect(isEnabled("debug", "warn")).toBe(false);
    expect(isEnabled("info", "error")).toBe(false);
  });

  it("drops everything when silent", () => {
    expect(isEnabled("error", "silent")).toBe(false);
  });
});

describe("createLogger", () => {
  const spies = {
    debug: vi.spyOn(console, "debug").mockImplementation(() => undefined),
    info: vi.spyOn(console, "info").mockImplementation(() => undefined),
    warn: vi.spyOn(console, "warn").mockImplementation(() => undefined),
    error: vi.spyOn(console, "error").mockImplementation(() => undefined),
  };
  afterEach(() => {
    for (const spy of Object.values(spies)) spy.mockClear();
  });

  it("prefixes the scope and passes fields to the matching console method", () => {
    createLogger("api", "debug").warn("save failed", { status: 500 });
    expect(spies.warn).toHaveBeenCalledWith("[api] save failed", { status: 500 });
    expect(spies.error).not.toHaveBeenCalled();
  });

  it("omits the fields argument when there are none", () => {
    createLogger("router", "debug").info("ready");
    expect(spies.info).toHaveBeenCalledWith("[router] ready");
  });

  it("filters by level", () => {
    const log = createLogger("x", "warn");
    log.debug("a");
    log.info("b");
    log.warn("c");
    log.error("d");
    expect(spies.debug).not.toHaveBeenCalled();
    expect(spies.info).not.toHaveBeenCalled();
    expect(spies.warn).toHaveBeenCalledTimes(1);
    expect(spies.error).toHaveBeenCalledTimes(1);
  });
});
