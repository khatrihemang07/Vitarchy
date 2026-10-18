import { usePrintings } from "@/api/finance";
import { errorMessage } from "../failureMessage";
import { FinanceNotice } from "../FinanceNotice";
import { PrintingLine } from "./PrintingLine";

/** Every Printing of one Transaction, fetched when first shown, with loading, error and empty states. */
export function PrintingsList({ transactionId, id }: { transactionId: number; id: string }) {
  const printings = usePrintings(transactionId, true);
  return (
    <div id={id} className="flex flex-col gap-2 py-1">
      {printings.isPending ? (
        <FinanceNotice kind="loading" inline title="Loading Printings…" />
      ) : printings.isError ? (
        <FinanceNotice kind="failure" inline title={`Could not load the Printings. ${errorMessage(printings.error)}`} onRetry={() => void printings.refetch()} />
      ) : printings.data.length === 0 ? (
        <FinanceNotice kind="empty" inline title="No Printings are stored for this Transaction." />
      ) : (
        <ul aria-label="Printings" className="flex flex-col gap-2">
          {printings.data.map((printing) => (
            <PrintingLine key={printing.id} printing={printing} />
          ))}
        </ul>
      )}
    </div>
  );
}
