-- Payment apps print Transactions too (ADR 0016, superseding 0014). A payment app's rows are
-- Printings on the Account that paid, so Payment records are retired: this converts what was
-- linked, reopens what was not, remembers which Account each instrument is, and drops them.

-- A Printing, and the Transaction it resolves to, may carry the time a source printed.
ALTER TABLE finance.printings    ADD COLUMN time time;
ALTER TABLE finance.transactions ADD COLUMN time time;

-- Which Account an instrument (how a payment app names the Account it paid from) is.
-- `instrument` is the printed text with runs of whitespace squashed and upper-cased.
CREATE TABLE finance.instrument_accounts (
    instrument text   PRIMARY KEY,
    account_id bigint NOT NULL REFERENCES finance.accounts (id) ON DELETE CASCADE
);

-- Linked records become Printings of their Transaction, from their payment history.
INSERT INTO finance.printings
    (transaction_id, statement_id, date, time, description, reference, source_id)
SELECT r.transaction_id, r.statement_id, r.occurred_at::date, r.occurred_at::time,
       r.description, r.utr, r.source_id
FROM finance.payment_records r
WHERE r.transaction_id IS NOT NULL
ORDER BY r.id
ON CONFLICT (statement_id, transaction_id) DO NOTHING;

-- Re-resolve the Transactions that gained one, as the server's resolve does: the new
-- Printing is the latest, so a gap field takes its value, and the description is replaced
-- only by one that strictly extends it (alphanumerics only, case folded).
UPDATE finance.transactions t
SET time      = COALESCE(t.time, p.time),
    reference = COALESCE(t.reference, p.reference),
    source_id = COALESCE(t.source_id, p.source_id),
    description = CASE
        WHEN length(lower(regexp_replace(p.description, '[^[:alnum:]]', '', 'g')))
             > length(lower(regexp_replace(t.description, '[^[:alnum:]]', '', 'g')))
         AND position(lower(regexp_replace(t.description, '[^[:alnum:]]', '', 'g'))
                      IN lower(regexp_replace(p.description, '[^[:alnum:]]', '', 'g'))) > 0
        THEN p.description ELSE t.description END
FROM finance.printings p
JOIN finance.payment_records r
  ON r.transaction_id = p.transaction_id AND r.statement_id = p.statement_id
WHERE t.id = p.transaction_id;

-- An instrument whose linked records all landed on one Account is remembered as that Account.
INSERT INTO finance.instrument_accounts (instrument, account_id)
SELECT upper(regexp_replace(btrim(r.instrument), '\s+', ' ', 'g')), min(t.account_id)
FROM finance.payment_records r JOIN finance.transactions t ON t.id = r.transaction_id
WHERE r.instrument IS NOT NULL AND btrim(r.instrument) <> ''
GROUP BY upper(regexp_replace(btrim(r.instrument), '\s+', ' ', 'g'))
HAVING count(DISTINCT t.account_id) = 1;

-- A payment history that had any unlinked record is read again: it goes back to unconfirmed
-- and is kept only as its original file, so the person confirms it again with a section per
-- instrument. The stored result held records, not sections, so it is cleared; the original
-- file stays and "Read again" fills it afresh.
UPDATE finance.statements s
SET status = 'unconfirmed', confirmed_at = NULL, outcome = 'crashed', kind = NULL,
    result = NULL, reader_version = NULL
WHERE s.kind = 'payment_history'
  AND EXISTS (SELECT 1 FROM finance.payment_records r
              WHERE r.statement_id = s.id AND r.transaction_id IS NULL);

DROP TABLE finance.payment_records;
