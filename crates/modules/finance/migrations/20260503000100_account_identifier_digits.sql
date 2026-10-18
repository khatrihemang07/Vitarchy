-- Accounts are matched on the trailing digits both masks show, and some cards print only 2.
-- A remembered identifier therefore holds those visible digits, not exactly 4.
ALTER TABLE finance.account_identifiers RENAME COLUMN last4 TO digits;
ALTER TABLE finance.account_identifiers DROP CONSTRAINT account_identifiers_last4_check;
ALTER TABLE finance.account_identifiers ADD CONSTRAINT account_identifiers_digits_check CHECK (digits ~ '^[0-9]{2,}$');
