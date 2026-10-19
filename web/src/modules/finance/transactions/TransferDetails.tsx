import type { TransferInfo } from "@/api/finance";
import { transferRungLabel, transferStatusLine } from "./transferCopy";
import { UnlinkTransferButton } from "./UnlinkTransferButton";

interface TransferDetailsProps {
  transfer: TransferInfo;
  direction: "debit" | "credit";
  names: ReadonlyMap<number, string>;
}

/** The inside of a Transfer's popover: the other side, anything still to arrive, how it was linked, Unlink. */
export function TransferDetails({ transfer, direction, names }: TransferDetailsProps) {
  const status = transferStatusLine(transfer);
  return (
    <>
      <div>
        <p className="font-medium">Transfer</p>
        <p className="text-sm text-muted-foreground">Money moving between your own Accounts, so it is not counted as spending or income.</p>
      </div>
      <dl className="flex flex-col gap-1 text-sm">
        <dt className="text-muted-foreground">{direction === "debit" ? "Paid into" : "Paid from"}</dt>
        <dd>
          <ul className="flex flex-col">
            {transfer.other_accounts.map((id) => (
              <li key={id}>{names.get(id) ?? "Account"}</li>
            ))}
          </ul>
        </dd>
      </dl>
      {status ? <p className="text-sm">{status}</p> : null}
      <p className="text-xs text-muted-foreground">{transferRungLabel(transfer.rung)}</p>
      <UnlinkTransferButton transferId={transfer.id} />
    </>
  );
}
