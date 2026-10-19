import { usePrintings } from "@/api/finance";
import { Notice } from "@/design";
import { errorMessage } from "../failureMessage";
import { PrintingLine } from "./PrintingLine";

/** Every Printing of one Transaction, fetched when first shown, with loading, error and empty states. */
export function PrintingsList({ transactionId }: { transactionId: number }) {
  const printings = usePrintings(transactionId, true);
  return (
    <div className="flex flex-col gap-2">
      {printings.isPending ? (
        <Notice kind="loading" inline title="Loading Printings…" />
      ) : printings.isError ? (
        <Notice kind="failure" inline title={`Could not load the Printings. ${errorMessage(printings.error)}`} onRetry={() => void printings.refetch()} />
      ) : printings.data.length === 0 ? (
        <Notice kind="empty" inline title="No Printings are stored for this Transaction." />
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
