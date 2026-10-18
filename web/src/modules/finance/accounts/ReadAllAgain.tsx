import { useState } from "react";
import { type ReadAgainItem, type ReadAllAgainCounts, useReadAllAgain } from "@/api/finance";
import { MotionButton } from "@/design";
import { errorMessage } from "../failureMessage";
import { FinanceNotice } from "../FinanceNotice";
import { ReadAllAgainPanel } from "./ReadAllAgainPanel";

/** The Read all again button of the Statements card and, once clicked, its combined preview. */
export function ReadAllAgain() {
  const readAll = useReadAllAgain();
  const [items, setItems] = useState<ReadAgainItem[] | null>(null);
  const [applied, setApplied] = useState<ReadAllAgainCounts | null>(null);
  return (
    <div className="flex flex-col gap-3">
      <div>
        <MotionButton
          variant="outline"
          size="sm"
          disabled={readAll.isPending}
          onClick={() => {
            setApplied(null);
            readAll.mutate(undefined, { onSuccess: setItems });
          }}
        >
          {readAll.isPending ? "Reading all…" : "Read all again"}
        </MotionButton>
      </div>
      {readAll.isError ? <FinanceNotice kind="failure" inline title={errorMessage(readAll.error)} /> : null}
      {items ? <ReadAllAgainPanel items={items} applied={applied} onApplied={setApplied} onClose={() => { setItems(null); }} /> : null}
    </div>
  );
}
