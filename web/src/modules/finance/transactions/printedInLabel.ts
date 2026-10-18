import { formatCount } from "../money";

/** "Printed in 2 Statements": shown on a Transaction that has more than one Printing. */
export function printedInLabel(printingCount: number): string {
  return `Printed in ${formatCount(printingCount)} Statements`;
}
