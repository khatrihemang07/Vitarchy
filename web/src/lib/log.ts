/**
 * Console logging for the web app. Local only: nothing is sent anywhere, and
 * this is the one file allowed to touch `console` (ESLint `no-console`), so
 * level filtering and the `[scope] message {fields}` shape stay uniform.
 *
 * Never pass secrets or setting values as fields: log keys, scopes, statuses.
 */

/** Severity of one log line. */
export type LogLevel = "debug" | "info" | "warn" | "error";
/** `silent` is a threshold only; nothing is logged at it. */
export type LogThreshold = LogLevel | "silent";
/** Structured detail on a line: keys, scopes and statuses, never values the person entered. */
export type LogFields = Record<string, unknown>;

/** A scoped logger, one method per [`LogLevel`]. */
export interface Logger {
  /** Detail for chasing a bug; shown in development by default. */
  debug: (message: string, fields?: LogFields) => void;
  /** A meaningful state change (a save, a startup step). */
  info: (message: string, fields?: LogFields) => void;
  /** Unexpected but handled (a rejected save, an unknown route). */
  warn: (message: string, fields?: LogFields) => void;
  /** Something failed and needs attention. */
  error: (message: string, fields?: LogFields) => void;
}

const RANK: Record<LogThreshold, number> = { debug: 0, info: 1, warn: 2, error: 3, silent: 4 };

/** Reads a user-supplied level (env var), falling back when it is missing or unknown. */
export function parseLogLevel(raw: string | undefined, fallback: LogThreshold): LogThreshold {
  return raw !== undefined && raw in RANK ? (raw as LogThreshold) : fallback;
}

/** Pure so the filtering rule is testable without a console. */
export function isEnabled(level: LogLevel, threshold: LogThreshold): boolean {
  return RANK[level] >= RANK[threshold];
}

/** Chatty while developing, quiet (warnings and errors) in a production build. */
const configuredThreshold = parseLogLevel(
  import.meta.env.VITE_LOG_LEVEL,
  import.meta.env.DEV ? "debug" : "warn",
);

/**
 * A logger tagged with `scope` (e.g. "api", "router"). `threshold` exists for
 * tests; the app uses the build-wide `VITE_LOG_LEVEL`.
 */
export function createLogger(scope: string, threshold: LogThreshold = configuredThreshold): Logger {
  const emit = (level: LogLevel) => (message: string, fields?: LogFields) => {
    if (!isEnabled(level, threshold)) return;
    const line = `[${scope}] ${message}`;
    if (fields) console[level](line, fields);
    else console[level](line);
  };
  return { debug: emit("debug"), info: emit("info"), warn: emit("warn"), error: emit("error") };
}
