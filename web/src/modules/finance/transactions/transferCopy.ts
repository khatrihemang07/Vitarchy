import type { TransferInfo } from "@/api/finance";
import { formatInr } from "../money";

type Rung = TransferInfo["rung"];

/** How many Account names the badge spells out before counting the rest. */
const NAMED = 2;

/** The badge of a Transfer row: "Transfer → " and the Accounts on the other side. */
export function transferBadgeLabel(info: TransferInfo, names: ReadonlyMap<number, string>): string {
  const named = info.other_accounts.map((id) => names.get(id) ?? "Account");
  const rest = named.length - NAMED;
  const shown = named.slice(0, NAMED).join(", ");
  return `Transfer → ${rest > 0 ? `${shown} +${rest.toString()}` : shown}`;
}

/** What is still to arrive, or what arrived beyond the debit; null for a Transfer that adds up. */
export function transferStatusLine(info: TransferInfo): string | null {
  if (info.partial) return `${formatInr(info.remaining_paise)} is still on its way and counts as Money out until it arrives.`;
  if (info.extra_paise > 0) return `${formatInr(info.extra_paise)} more arrived than was sent and counts as Money in.`;
  return null;
}

const RUNGS: Record<Rung, string> = {
  reference: "Matched by its reference",
  account_number: "Matched by the Account number in its description",
  bill_payment: "Matched as a card bill payment",
  unique_amount: "Matched by its amount, the only one that day",
  manual: "Linked by you",
};

/** How a Transfer came to be linked, for the popover. */
export function transferRungLabel(rung: Rung): string {
  return RUNGS[rung];
}
