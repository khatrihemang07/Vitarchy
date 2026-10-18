# A Transaction keeps every Printing

Statements overlap, so one Transaction is printed by several files, and the Printings differ: SBI's PDF cuts a narration at 50 characters where its XLSX prints it whole, and an ICICI history PDF and its monthly Statement each carry details the other lacks. Keeping only the first stored Printing drops whatever a later, richer one adds. So every Statement's Printing of a Transaction is kept, and the Transaction's own columns are the **resolved view** derived from its Printings.

The resolved view fills gaps (balance, reference, id and foreign amount come from the first Printing that has one) and shows the description of the first Printing, replaced only by a later one that strictly extends it: the shown text, compared as lowercase letters and digits, is contained in the new text and the new text is longer. Otherwise the first text stays, and every Printing remains viewable. Amount, date and direction never change after a Transaction is created. A Transaction counts as no longer found only when every one of its Printings has been marked gone by a Read again. Nothing is ever deleted.

## Considered Options

- **First Printing wins (the old behaviour).** Rejected: detail a later Statement prints is lost for good, and the rule cannot be improved later because the other Printings were never kept.
- **Ask the person per Transaction which Printing to keep.** Rejected: hundreds of overlapping rows per import, and the answer is almost always "the fuller one".
- **Merge into the Transaction and drop the other Printings.** Rejected: the merge rule cannot then be changed or re-run, and a Read again cannot tell which Statement still prints a row.

## Consequences

- The merged view is recomputed from the Printings whenever one is stored or marked, so a better rule can be applied to existing data without a re-import.
- Matching a new row to a stored Transaction compares it with the resolved view, through the one matcher for confirm, preview and Read again.
- A Read again marks only its own Statement's Printings, and a row it finds in a Transaction another Statement stored gains a Printing instead of a second Transaction.
- A strict-extension rule is cautious: when a Statement's text differs from the fuller one in its last characters, the first text stays. Every Printing is still shown, so nothing is hidden.
