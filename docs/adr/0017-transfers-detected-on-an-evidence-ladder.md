# Transfers are detected on an evidence ladder

Money moving between the person's own Accounts (a savings Account paying a card bill, a monthly sum to another bank, a loan instalment) is neither spending nor income, yet every Statement prints it twice, once as a debit and once as a credit. Left alone it inflates both totals. A **Transfer** links one Money-out Transaction to one or more Money-in Transactions on other Accounts, and everything that counts Money in or Money out leaves Transfers out.

Like ADR 0012's matcher for Transactions, Transfers are found by an **evidence ladder**, strongest first, over the own Accounts that are not hidden. Each rung is a pure function of the Transactions' resolved views and every Printing's text (ADR 0015); a Transaction used by one rung is not used again, and a debit or credit that fits several partners is left alone, because a wrong link hides spending while a missed one only shows a Transfer as two rows.

1. **Reference.** The debit's reference (8 or more characters) appears as a whole token in any Printing's description, or is the reference, of a credit on another Account. Exact amount, within a day.
2. **Account number.** A run of 8 or more digits in the debit's text ends with the digits another Account is known by (4 or more, from its mask and remembered identifiers). Exact amount, the same day.
3. **Bill payment.** A debit whose text names a card-bill payee, and credits on card Accounts that read as payment receipts, within a day. The payee and receipt markers are data (`transfers/markers.toml`). A set of up to four receipts settles a payment when it adds up to the debit or up to **₹50 more**; an exact set beats one with extra, and a payment is assigned only when one set fits it and no other payment could take the same receipts on equal or better terms. Receipts left over join the one payment that still has room (a **partial** Transfer), and later runs add receipts to a Transfer that is still partial.
4. **Unique amount.** Exact amount, within a day, the debit and the credit each the only candidate for the other among all Transactions of the own, visible Accounts, including ones already linked.

Only the third rung has any tolerance, because only a card-bill payment can arrive as cashback-adjusted receipts; elsewhere a difference of a rupee means two different payments. A Transfer can be **1:N** because one bill payment is routinely split across cards, and **partial** while some card's Statement is not imported yet: the missing part counts as Money out until it arrives, and an **extra** counts as Money in.

**Unlinking is remembered.** Unlinking records every (debit, credit) pair, and detection never links a remembered pair again, however the data is read again or merged. Linking by hand (`manual`) ignores that memory and has no uniqueness rule, but needs different Accounts, opposite directions, no Transaction already in a Transfer, and credits adding up to no more than the debit plus ₹50.

**Hidden Accounts are outside it.** An Account the person hides (a loan that is funded from outside Vitarchy, say) takes no part: its Transactions are not candidates. Hiding it dissolves all of its Transfers, manual ones included, without recording any rejection (a Transfer where it held the debit goes whole; where it held a credit, that credit is unlinked and a Transfer left with none dissolves). Showing it again re-runs detection over its dates, so pairs that were dissolved can link again.

**Only bill payments complete by themselves.** A partial Transfer grows automatically only when it was linked on the bill-payment rung; the exact-amount rungs are complete the moment they link, and a manual Transfer is left as the person made it. Manual links may exceed the debit by up to ₹50 only when every credit is on a credit card, and otherwise must add up to at most the debit (partial is allowed); the same rule filters the candidates offered for a hand-made link. A Transaction that a Read again no longer finds is never linked.

**Thresholds, and why.**
- A reference must be **at least 8 characters**: shorter ones (cheque numbers, small ids) repeat across unrelated rows.
- A number in a description counts as an Account number only as a run of **at least 8 digits**, and only against **known digits of at least 4**: a run of fewer digits is a date or an amount, and one or two trailing digits fit many Accounts.
- At most **4 receipts** are combined into one payment, and a window with more than **24 candidate receipts** is not judged at all: the search is exponential in the pool, and a window that crowded is ambiguous anyway.
- The day windows (a day for references, receipts and amounts; the same day for Account numbers) follow how banks date the two sides of one movement.

**When it runs.** After a Statement is confirmed or read again, after Accounts are merged and after an Account is shown, over the dates that changed plus 3 days either side (reading 4 more so a Transaction at the edge still meets its rivals), and once as a backfill at server start, guarded by a marker row, for Transactions stored before Transfers existed. A failed run is logged and never undoes the change that caused it. Each run logs its trigger, the Transfers it found per rung and how many it left ambiguous, never an amount or a description.

## Considered Options

- **Link by amount alone.** Rejected: a bare amount collides with same-amount instalments and purchases (a loan credit shares its amount with unrelated SIP debits), which is why the weaker rungs need uniqueness and the stronger ones need evidence.
- **A tolerance on every rung.** Rejected: only card-bill payments differ by small credits; elsewhere it would link different payments.
- **Only one credit per Transfer.** Rejected: a bill payment is split across cards.
- **Ask the person to confirm every link.** Rejected: dozens a month, nearly all certain. The person unlinks the odd wrong one instead, and it stays unlinked.

## Consequences

- Money in and Money out, and totals built on them, can leave Transfers out. A partial Transfer's remainder is Money out and an extra is Money in, both derived from the amounts and never stored.
- A Transfer follows its Transactions: it dissolves when its debit or all its credits are deleted, and a merge of duplicate Accounts moves links onto the surviving Transaction.
- Which Transfers link can depend on what was imported when, because uniqueness is judged against what is stored; a later run links what the earlier one left ambiguous only if the rivals are gone.
- The weak rung can still link a loan's outside deposit to an unrelated debit of the same amount; hiding that Account is the remedy.
