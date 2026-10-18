import { ChevronDown } from "lucide-react";
import { useId, useState } from "react";
import type { AccountSummary, SectionPreview, StatementKind } from "@/api/finance";
import { cn, Expand } from "@/design";
import { formatPeriod } from "../dates";
import { formatCount, formatInr } from "../money";
import { previewCountLine } from "./importCopy";
import type { ChoiceDraft } from "../sectionChoice";
import { sectionHeading } from "../statementCopy";
import { AccountPicker } from "./AccountPicker";
import { ChecksList } from "./ChecksList";
import { StatementRowsTable } from "../StatementRowsTable";

interface SectionViewProps {
  section: SectionPreview;
  /** The Statement's kind: a Payment history names its sections by Instrument. */
  statementKind: StatementKind | null | undefined;
  accounts: readonly AccountSummary[];
  draft: ChoiceDraft;
  onDraft: (next: ChoiceDraft) => void;
  /** A confirmed Statement shows no picker. */
  locked: boolean;
}

/** One Account section of a Statement: what the reader found, where it goes, its Checks and rows. */
export function SectionView({ section, statementKind, accounts, draft, onDraft, locked }: SectionViewProps) {
  const tableId = useId();
  const [open, setOpen] = useState(false);
  const countLine = previewCountLine(section);
  const heading = sectionHeading(statementKind, section.proposed);
  return (
    <section aria-label={heading} className="flex flex-col gap-4 rounded-xl border bg-muted/30 p-4">
      <div className="flex flex-col gap-1">
        <h3 className="font-display text-base font-semibold break-words">{heading}</h3>
        <p className="text-sm text-muted-foreground">
          {formatPeriod(section.period)} · Debits {formatInr(section.total_debit_paise)} · Credits {formatInr(section.total_credit_paise)}
        </p>
      </div>
      {locked ? null : <AccountPicker section={section} accounts={accounts} draft={draft} onChange={onDraft} />}
      <ChecksList checks={section.checks} />
      <p className="text-sm">
        <span className="font-medium">{countLine.lead}</span>
        <span className="text-muted-foreground">{countLine.rest}</span>
      </p>
      <div className="flex flex-col gap-3">
        <button
          type="button"
          aria-expanded={open}
          aria-controls={tableId}
          onClick={() => { setOpen((v) => !v); }}
          className="flex w-fit items-center gap-1.5 rounded-md text-sm font-medium focus-ring"
        >
          <ChevronDown aria-hidden="true" className={cn("size-4 transition-transform", open && "rotate-180")} />
          {open ? "Hide" : "Show"} {formatCount(section.transaction_count)} Transactions
        </button>
        <Expand open={open} id={tableId}>
          <StatementRowsTable rows={section.transactions} caption={`Transactions in ${heading}`} />
        </Expand>
      </div>
    </section>
  );
}
