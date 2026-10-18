# Payment apps print Transactions too

Supersedes [ADR 0014](0014-payment-records-link-to-transactions.md).

ADR 0014 made a payment app's rows Payment records that only added detail to a bank Transaction, so a month the person's bank Statements did not cover showed nothing of their PhonePe spending, and the person's first upload said its payments "will wait". The person sees a payment app as a source of Transactions in its own right. With Printings in place (ADR 0015) it can be one without counting anything twice: each row of a payment app's file is a **Printing** of a Transaction on the Account that paid, and the bank's row for the same payment later joins that Transaction (or the other way round).

**Instruments.** A payment app names the Account it paid from by an **Instrument** (`XXXXXX4321`, `ICICI R`), never by what kind of Account it is. The reader returns one section per Instrument, and each section is confirmed to an Account like any Statement section. Vitarchy proposes the Account in this order: the one the person confirmed for that Instrument before (`instrument_accounts`), then the Accounts that show the same visible digits and the same issuer when the Instrument names one, across kinds; otherwise the person chooses or creates one, giving its kind. The choice is remembered. A new Account is still never created twice: a "new" Account whose issuer, kind and digits match an existing one is that Account.

**One matcher, one more rung.** The ladder that tells whether two rows are one Transaction (ADR 0012) gains two changes:

- A matching reference (a UTR) now bridges a day, as an id or a balance does (the app and the bank may date a payment a day apart).
- A last rung joins a payment app's row to a bank or card row of the same Account that agree on amount, direction and a date within a day, but only when each has **exactly one** such partner. Two same-amount payments in the window merge nothing, because a wrong merge hides a payment while a missed one only shows it twice. It never joins two rows of the same kind of file, nor two rows whose references (or ids, within one kind of file) are both present and differ.

Ids from a bank and from a payment app live in different namespaces, so they neither agree nor disagree; a row is compared with each Printing of a Transaction as well as with its resolved view, so a payment app's id still identifies a Transaction whose shown id came from a bank. A Printing's origin is the kind of its Statement (derived from the Statement kind by a join, not stored, so there is one source of truth). Matching uses every Printing of a Transaction, gone ones included, on purpose: a row that comes back revives its Transaction instead of storing a second one. Printings and Transactions gain a nullable `time`, filled like the other gap fields.

## Considered Options

- **Keep Payment records (ADR 0014).** Rejected: payments the bank Statements do not cover stay invisible, and the "waiting" state is a second kind of record to explain.
- **A PhonePe Account.** Still rejected: the money never sat in PhonePe.
- **Always merge a lone amount-and-date match, or merge the nearest.** Rejected: two payments of one amount a day apart are common, and a silent wrong merge is worse than a visible duplicate.

## Consequences

- Payment records, the linking code and "unlinked payment records" are gone. A migration turned each linked record into a Printing of its Transaction (with time, UTR as reference, the app's id as id), remembered the Account of each Instrument whose linked records agreed on one, and sent every payment history that still had an unlinked record back to unconfirmed with its stale result cleared (the original file is kept, and a Read again fills it), so the person confirms it again with a section per Instrument. A history that was fully linked stays confirmed.
- Uploading a payment app's file before or after the bank's gives the same Transactions.
- An unconfirmed Statement that already printed a Transaction cannot be discarded.
