import type { ReadAgainPreview } from "@/api/finance";
import { formatMask } from "../mask";
import { sectionChangesLine } from "./readAgainCopy";
import { accountKindLabel } from "../statementCopy";
import { StatementRowsTable } from "../StatementRowsTable";

/** What a Read again would change: per Account section, rows to add, rows unchanged and rows no longer found. */
export function ReadAgainChanges({ preview }: { preview: ReadAgainPreview }) {
  return (
    <div className="flex flex-col gap-4">
      {preview.sections.map((changes) => {
        const proposed = preview.statement.sections.find((s) => s.position === changes.position)?.proposed;
        const heading = [proposed?.issuer, accountKindLabel(proposed?.kind), formatMask(proposed?.mask)].filter(Boolean).join(" · ");
        return (
          <section key={changes.position} aria-label={heading} className="flex flex-col gap-2">
            <h4 className="text-sm font-semibold break-words">{heading}</h4>
            <p className="text-sm text-muted-foreground">
              {sectionChangesLine(changes)}
            </p>
            {changes.added.length > 0 ? <StatementRowsTable rows={changes.added} caption={`Transactions to add to ${heading}`} /> : null}
            {changes.new_printings.length > 0 ? (
              <StatementRowsTable rows={changes.new_printings} caption={`Stored Transactions that gain a Printing in ${heading}`} />
            ) : null}
          </section>
        );
      })}
    </div>
  );
}
