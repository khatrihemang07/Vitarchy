import type { Direction } from "@/api/finance";

/** Indian digit grouping: the last three digits, then pairs ("1234567" becomes "12,34,567"). */
function groupIndian(digits: string): string {
  if (digits.length <= 3) return digits;
  const head = digits.slice(0, -3).replace(/\B(?=(\d{2})+(?!\d))/g, ",");
  return `${head},${digits.slice(-3)}`;
}

/** An amount in paise as rupees with Indian grouping: 12345678 becomes "₹1,23,456.78". A negative amount gets a leading minus. */
export function formatInr(paise: number): string {
  const abs = Math.abs(Math.round(paise));
  const rupees = String(Math.trunc(abs / 100));
  const fraction = String(abs % 100).padStart(2, "0");
  return `${paise < 0 ? "−" : ""}₹${groupIndian(rupees)}.${fraction}`;
}

/** Whether money came in rather than went out. */
export function isCredit(direction: Direction): boolean {
  return direction === "credit";
}

/** A Transaction's amount with its direction: "−₹500.00" for a debit, "+₹500.00" for a credit. */
export function formatSignedAmount(paise: number, direction: Direction): string {
  return `${isCredit(direction) ? "+" : "−"}${formatInr(Math.abs(paise))}`;
}

/** A balance after a row in INR, or nothing when the Statement did not print one. */
export function formatBalance(paise: number | null | undefined): string {
  return paise === null || paise === undefined ? "" : formatInr(paise);
}

const countFormat = new Intl.NumberFormat("en-IN");

/** A count with Indian grouping ("1,23,456"). */
export function formatCount(n: number): string {
  return countFormat.format(n);
}
