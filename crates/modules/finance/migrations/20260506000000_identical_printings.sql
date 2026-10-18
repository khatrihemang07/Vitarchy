-- A Statement that prints a Transaction exactly as a kept, live Printing adds no new one
-- (ADR 0015). Remove existing Printings identical to an earlier live Printing of the same
-- Transaction, keeping the earliest; the shown view cannot change. A gone later Printing
-- identical to an earlier live one goes too, so its Statement's gone record is lost.
CREATE TEMP TABLE identical_printings ON COMMIT DROP AS
SELECT p.id, p.transaction_id
FROM finance.printings p
WHERE EXISTS (
    SELECT 1 FROM finance.printings q
    WHERE q.transaction_id = p.transaction_id AND q.id < p.id AND q.gone_at IS NULL
      AND q.date = p.date AND q.description = p.description
      AND q.balance_paise IS NOT DISTINCT FROM p.balance_paise
      AND q.reference IS NOT DISTINCT FROM p.reference
      AND q.source_id IS NOT DISTINCT FROM p.source_id
      AND q.foreign_amount IS NOT DISTINCT FROM p.foreign_amount
      AND q.foreign_currency IS NOT DISTINCT FROM p.foreign_currency
);

DELETE FROM finance.printings WHERE id IN (SELECT id FROM identical_printings);

-- Re-resolve "no longer found": set only when every remaining Printing is gone.
UPDATE finance.transactions t
SET no_longer_found_at = CASE
    WHEN EXISTS (SELECT 1 FROM finance.printings p WHERE p.transaction_id = t.id AND p.gone_at IS NULL)
    THEN NULL ELSE COALESCE(t.no_longer_found_at, now()) END
WHERE t.id IN (SELECT transaction_id FROM identical_printings);
