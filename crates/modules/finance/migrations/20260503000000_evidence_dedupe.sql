-- Duplicates are now decided by an evidence ladder (id, balance, reference, description)
-- over candidate rows, not by a stored fingerprint (ADR 0012). Candidates are found by
-- Account, amount, direction and date.
DROP INDEX IF EXISTS finance.transactions_fingerprint_idx;
ALTER TABLE finance.transactions DROP COLUMN fingerprint;
CREATE INDEX transactions_candidates_idx
    ON finance.transactions (account_id, amount_paise, direction, date);
