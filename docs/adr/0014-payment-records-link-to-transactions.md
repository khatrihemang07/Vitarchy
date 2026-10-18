# Payment records link to Transactions

A payment app's Payment history (PhonePe now) describes the same money that the person's bank and card Statements already list. Treating it as an Account would count every UPI payment twice. Instead, each Payment record is extra detail (the exact time, the other party, the app's ids) attached to at most one Transaction. The link is made first by UTR, which banks print inside UPI narrations (39 of 39 matched in the sample), and then by amount, direction and date. A Payment record whose Account has not been imported yet waits, and links when that Statement is confirmed.

## Considered Options

- **A PhonePe Account.** Rejected: it double counts spending, and the money never sat in PhonePe.
- **Ignore payment apps.** Rejected: they hold the only reliable counterparty names and times.

## Consequences

- Linking is a pure function, run when a Statement or Payment history is confirmed.
- A Payment record never changes an amount, and its Transaction stays exactly as printed.
