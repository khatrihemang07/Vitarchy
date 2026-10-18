import type { CheckView, StatementPreview } from "@/api/finance";

/** The kinds of Account, as the person names them. */
export const ACCOUNT_KINDS = [
  { value: "savings", label: "Savings account" },
  { value: "credit_card", label: "Credit card" },
  { value: "loan", label: "Loan" },
] as const;

/** "Credit card" for `credit_card`; an unknown kind is shown as it came. */
export function accountKindLabel(kind: string | null | undefined): string {
  if (!kind) return "Unknown kind";
  return ACCOUNT_KINDS.find((k) => k.value === kind)?.label ?? kind;
}

/** "PDF", "XLSX": the file format in capitals. */
export function formatLabel(format: string | null | undefined): string {
  return format ? format.toUpperCase() : "Unknown format";
}

/** What kind of file it is, in the Glossary's words. */
export function statementKindLabel(kind: string | null | undefined): string {
  return kind === "payment_history" ? "Payment history" : "Statement";
}

const STAGE_WORDS: Record<string, string> = {
  open: "opening the file",
  read: "reading its pages",
  locate: "finding the table of Transactions",
  segment: "splitting the table into rows",
  normalize: "reading dates and amounts",
};

/** How a file that was not read is explained: a title and what to do. */
export interface UnreadCopy {
  title: string;
  body: string;
}

/** The explanation for a Statement the reader did not read, or null for one it read. */
export function unreadCopy(preview: Pick<StatementPreview, "outcome" | "diagnostics">): UnreadCopy | null {
  switch (preview.outcome) {
    case "read":
      return null;
    case "locked":
      return { title: "No saved password opened this file", body: "Save the password for this Statement, then try again." };
    case "unsupported":
      return {
        title: "This type of file is not supported",
        body: "Statements can be PDF, CSV, XLS or XLSX files. The reader could not open this one.",
      };
    case "no_table": {
      const stage = preview.diagnostics.stage ? STAGE_WORDS[preview.diagnostics.stage] : undefined;
      return {
        title: "No table of Transactions was found",
        body: `The file opened, but the reader stopped while ${stage ?? "looking for Transactions"}. You can discard it and try again after the reader improves.`,
      };
    }
    default:
      return {
        title: "The reader failed on this file",
        body: "Something went wrong while reading it. Use Read again, or discard it.",
      };
  }
}

const CHECK_NAMES: Record<string, string> = {
  balance_chain: "Balances",
  summary_totals: "Totals",
  row_counts: "Row count",
};

const CHECK_WORDS: Record<string, Record<string, string>> = {
  balance_chain: {
    passed: "Each balance follows from the one before it.",
    failed: "Some balances do not follow from the one before. A row may be missing or misread.",
    skipped: "No balances were printed to follow.",
  },
  summary_totals: {
    passed: "The totals match the Statement's printed summary.",
    failed: "The totals differ from the Statement's printed summary.",
    skipped: "The Statement printed no summary to compare with.",
  },
  row_counts: {
    passed: "The number of rows matches the count printed.",
    failed: "The number of rows differs from the count printed.",
    skipped: "No row count was printed to compare with.",
  },
};

/** A Check in plain language: its name and what its outcome means. */
export function checkCopy(check: Pick<CheckView, "name" | "outcome">): { name: string; text: string } {
  return {
    name: CHECK_NAMES[check.name] ?? check.name,
    text: CHECK_WORDS[check.name]?.[check.outcome] ?? check.outcome,
  };
}

/** Whether any Check failed, which warns without stopping a confirm. */
export function hasFailedCheck(checks: readonly Pick<CheckView, "outcome">[]): boolean {
  return checks.some((c) => c.outcome === "failed");
}
