-- What the helper recognised in a Payment record's funding account, so linking only
-- considers Accounts that agree with it (ADR 0014).
ALTER TABLE finance.payment_records
    ADD COLUMN instrument_issuer text,
    ADD COLUMN instrument_digits text;
-- Records stored before this carry only the printed instrument; its trailing digits are the
-- same thing the helper now reports.
UPDATE finance.payment_records
SET instrument_digits = substring(instrument FROM '([0-9]{2,})$')
WHERE instrument IS NOT NULL;
