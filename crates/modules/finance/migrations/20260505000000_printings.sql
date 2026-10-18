-- A Transaction keeps every Statement's Printing of it (ADR 0015). `finance.transactions` stays
-- as the resolved view, always derived from its Printings; amount, date and direction never
-- change after the Transaction is created.
CREATE TABLE finance.printings (
    id               bigserial   PRIMARY KEY,
    transaction_id   bigint      NOT NULL REFERENCES finance.transactions (id),
    statement_id     bigint      NOT NULL REFERENCES finance.statements (id),
    date             date        NOT NULL,
    description      text        NOT NULL,
    balance_paise    bigint,
    reference        text,
    source_id        text,
    foreign_amount   text,
    foreign_currency text,
    -- Set when a Read again of the Statement no longer finds this Printing; never deleted.
    gone_at          timestamptz,
    created_at       timestamptz NOT NULL DEFAULT now(),
    UNIQUE (statement_id, transaction_id)
);
CREATE INDEX printings_transaction_idx ON finance.printings (transaction_id);

-- One Printing per existing Transaction, from its own columns and Statement.
INSERT INTO finance.printings
    (transaction_id, statement_id, date, description, balance_paise, reference, source_id,
     foreign_amount, foreign_currency, gone_at, created_at)
SELECT id, statement_id, date, description, balance_paise, reference, source_id,
       foreign_amount, foreign_currency, no_longer_found_at, created_at
FROM finance.transactions
ORDER BY id;
