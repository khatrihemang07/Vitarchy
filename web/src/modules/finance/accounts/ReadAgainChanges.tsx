import type { ReadAgainPreview } from "@/api/finance";
import { formatCount } from "../money";
import { formatMask } from "../mask";
import { accountKindLabel } from "../statementCopy";
import { StatementRowsTable } from "../StatementRowsTable";

/** What a Read again would change: per Account section, rows to add, rows unchanged and rows no longer found. */
export function ReadAgainChanges({ preview }: { preview: ReadAgainPreview }) {
  const records = preview.payment_records;
  if (records) {
    return (
      <p className="text-sm">
        {formatCount(records.add_count)} new Payment records, {formatCount(records.unchanged_count)} unchanged.
      </p>
    );
  }
  return (
    <div className="flex flex-col gap-4">
      {preview.sections.map((changes) => {
        const proposed = preview.statement.sections.find((s) => s.position === changes.position)?.proposed;
        const heading = [proposed?.issuer, accountKindLabel(proposed?.kind), formatMask(proposed?.mask)].filter(Boolean).join(" · ");
        return (
          <section key={changes.position} aria-label={heading} className="flex flex-col gap-2">
            <h4 className="text-sm font-semibold break-words">{heading}</h4>
            <p className="text-sm text-muted-foreground">
              {formatCount(changes.add_count)} to add, {formatCount(changes.unchanged_count)} unchanged,{" "}
              {formatCount(changes.no_longer_found_count)} no longer found (marked, never deleted).
            </p>
            {changes.added.length > 0 ? <StatementRowsTable rows={changes.added} caption={`Transactions to add to ${heading}`} /> : null}
          </section>
        );
      })}
    </div>
  );
}
