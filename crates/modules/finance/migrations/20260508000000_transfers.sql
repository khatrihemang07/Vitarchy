-- Transfers: money moving between two of the person's own Accounts (ADR 0017). One debit
-- Transaction and one or more credit Transactions on other Accounts.

CREATE TABLE finance.transfers (
    id         bigserial   PRIMARY KEY,
    debit_id   bigint      NOT NULL UNIQUE REFERENCES finance.transactions (id) ON DELETE CASCADE,
    rung       text        NOT NULL CHECK (rung IN ('reference', 'account_number', 'bill_payment', 'unique_amount', 'manual')),
    created_at timestamptz NOT NULL DEFAULT now()
);

-- A credit belongs to at most one Transfer.
CREATE TABLE finance.transfer_credits (
    transfer_id bigint NOT NULL REFERENCES finance.transfers (id) ON DELETE CASCADE,
    credit_id   bigint NOT NULL UNIQUE REFERENCES finance.transactions (id) ON DELETE CASCADE,
    PRIMARY KEY (transfer_id, credit_id)
);

-- A Transfer left with no credit (they were deleted or folded away) dissolves.
CREATE FUNCTION finance.dissolve_empty_transfer() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    DELETE FROM finance.transfers t
    WHERE t.id = OLD.transfer_id
      AND NOT EXISTS (SELECT 1 FROM finance.transfer_credits c WHERE c.transfer_id = t.id);
    RETURN NULL;
END;
$$;

CREATE TRIGGER transfer_credits_dissolve
    AFTER DELETE ON finance.transfer_credits
    FOR EACH ROW EXECUTE FUNCTION finance.dissolve_empty_transfer();

-- Pairs the person unlinked: detection never links them again.
CREATE TABLE finance.transfer_rejections (
    debit_id  bigint NOT NULL REFERENCES finance.transactions (id) ON DELETE CASCADE,
    credit_id bigint NOT NULL REFERENCES finance.transactions (id) ON DELETE CASCADE,
    PRIMARY KEY (debit_id, credit_id)
);

-- A hidden Account's Transactions leave the Transactions page, and it takes no part in Transfers.
ALTER TABLE finance.accounts ADD COLUMN hidden boolean NOT NULL DEFAULT false;

-- One-time jobs that have run, so each runs once (the Transfer backfill at server start).
CREATE TABLE finance.markers (
    name    text        PRIMARY KEY,
    done_at timestamptz NOT NULL DEFAULT now()
);
