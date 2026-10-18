import { type FinanceFailure, toFinanceFailure } from "@/api/finance";

/** Words for a failed Finance request, fit to show the person. */
export function failureMessage(failure: FinanceFailure): string {
  switch (failure.kind) {
    case "server_unreachable":
      return "Could not reach the Vitarchy server.";
    case "reader_failed":
      return "The statement reader failed. Check that it is installed, then try again.";
    case "statement_not_readable":
      return "This file was not read, so it cannot be confirmed yet. Save its password or use Read again first.";
    case "new_account_required":
      return "An Account was not recognised. Choose New Account and give its issuer, kind and name.";
    case "account_ambiguous":
      return "Several Accounts fit. Choose which one this belongs to.";
    case "statement_sections_changed":
      return "The file now lists different Accounts than when it was confirmed, so nothing was changed.";
    case "statement_sections_unplaced":
      return "This file lists an Account that cannot be matched to your Transactions, so nothing was changed.";
    case "confirm_in_progress":
      return "Another confirm is running. Wait a moment and try again.";
    case "statement_confirmed":
      return "This Statement is already confirmed.";
    case "statement_has_rows":
      return "This file's rows are already part of your Transactions as Printings, so it cannot be discarded. Confirm it instead.";
    case "statement_not_found":
      return "This Statement no longer exists. It may have been discarded.";
    case "upload_too_large":
      return "That upload is too large. Upload fewer files at once.";
    case "no_files":
      return "Choose at least one file to upload.";
    case "other":
      return failure.message;
  }
}

/** Words for whatever a failed Finance query or mutation threw. */
export function errorMessage(error: unknown): string {
  return failureMessage(toFinanceFailure(error));
}
