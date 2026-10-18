import { RequestError } from "../callApi";

/** Why a Finance request failed, in terms the screen can act on. */
export type FinanceFailure =
  /** The Vitarchy server itself could not be reached. */
  | { kind: "server_unreachable" }
  /** The file was not read, so it cannot be confirmed or compared. */
  | { kind: "statement_not_readable" }
  /** A section's Account was not recognised: the person must name a new one. */
  | { kind: "new_account_required" }
  /** Several Accounts fit a section: the person must choose. */
  | { kind: "account_ambiguous" }
  /** A Read again now finds different Accounts than were confirmed. */
  | { kind: "statement_sections_changed" }
  /** A Read again cannot tell which Account a section of the file belongs to. */
  | { kind: "statement_sections_unplaced" }
  /** Another confirm is running. */
  | { kind: "confirm_in_progress" }
  | { kind: "statement_confirmed" }
  /** Its rows are already Printings of Transactions, so it cannot be discarded. */
  | { kind: "statement_has_rows" }
  | { kind: "statement_not_found" }
  | { kind: "upload_too_large" }
  | { kind: "no_files" }
  | { kind: "reader_failed" }
  | { kind: "other"; message: string };

const CODED = new Set([
  "statement_not_readable",
  "new_account_required",
  "account_ambiguous",
  "statement_sections_changed",
  "statement_sections_unplaced",
  "confirm_in_progress",
  "statement_confirmed",
  "statement_has_rows",
  "statement_not_found",
  "upload_too_large",
  "no_files",
]);

/** The failure a server `code` stands for, or null when the code is not one of ours. */
export function failureFromCode(code: unknown): FinanceFailure | null {
  return typeof code === "string" && CODED.has(code) ? ({ kind: code } as FinanceFailure) : null;
}

/** Classifies any thrown error from a Finance request by the problem's `code`. */
export function toFinanceFailure(error: unknown): FinanceFailure {
  if (!(error instanceof RequestError)) return { kind: "other", message: "Something went wrong." };
  const known = failureFromCode(error.problem?.code);
  if (known) return known;
  if (error.kind === "unreachable") {
    return error.status === 502 ? { kind: "reader_failed" } : { kind: "server_unreachable" };
  }
  return { kind: "other", message: error.message };
}
