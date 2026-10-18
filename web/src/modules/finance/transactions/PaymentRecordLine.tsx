import { Link2 } from "lucide-react";
import type { LinkedPaymentRecord } from "@/api/finance";
import { formatDateTime } from "../dates";

/** The Payment record linked to a Transaction: when, with whom, and its UTR. */
export function PaymentRecordLine({ record }: { record: LinkedPaymentRecord }) {
  return (
    <p className="mt-1 flex flex-wrap items-center gap-x-2 text-xs text-muted-foreground">
      <Link2 aria-hidden="true" className="size-3.5 shrink-0 text-module" />
      <span className="sr-only">Payment record:</span>
      <span>{formatDateTime(record.occurred_at)}</span>
      <span className="break-words">{record.description}</span>
      {record.utr ? <span className="tabular-nums">UTR {record.utr}</span> : null}
    </p>
  );
}
