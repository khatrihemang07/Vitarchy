import type { StatementPreview } from "@/api/finance";
import { formatCount } from "../money";

/** What a Payment history holds: its Payment records and how many link to a Transaction now. */
export function PaymentHistorySummary({ history }: { history: NonNullable<StatementPreview["payment_history"]> }) {
  const waiting = history.record_count - history.would_link_now;
  return (
    <div className="flex flex-col gap-1 rounded-xl border bg-muted/30 p-4 text-sm">
      <p>
        <span className="font-medium">{formatCount(history.record_count)} Payment records</span>
        <span className="text-muted-foreground">. {formatCount(history.would_link_now)} link to a Transaction now.</span>
      </p>
      <p className="text-muted-foreground">
        {waiting > 0
          ? `${formatCount(waiting)} will wait until a Statement holds the matching Transaction.`
          : "Every Payment record has its Transaction."}
      </p>
    </div>
  );
}
