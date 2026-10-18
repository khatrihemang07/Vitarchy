CREATE SCHEMA IF NOT EXISTS finance;

-- A savings account, credit card or loan at one issuer. `last4` is derived from `mask`
-- by the server and is what Statements are matched on.
CREATE TABLE finance.accounts (
    id         bigserial   PRIMARY KEY,
    issuer     text        NOT NULL,
    kind       text        NOT NULL CHECK (kind IN ('savings', 'credit_card', 'loan')),
    mask       text,
    last4      text        CHECK (last4 ~ '^[0-9]{4}$'),
    name       text        NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

-- Extra (issuer, kind, last 4) triples that also mean this Account: remembered when the
-- person reassigns a Statement's proposed Account.
CREATE TABLE finance.account_identifiers (
    id         bigserial PRIMARY KEY,
    account_id bigint    NOT NULL REFERENCES finance.accounts (id) ON DELETE CASCADE,
    issuer     text      NOT NULL,
    kind       text      NOT NULL CHECK (kind IN ('savings', 'credit_card', 'loan')),
    last4      text      NOT NULL CHECK (last4 ~ '^[0-9]{4}$'),
    UNIQUE (account_id, issuer, kind, last4)
);

-- One uploaded file. `result` is the reader's full JSON result; Transactions are written
-- from it only when the person confirms.
CREATE TABLE finance.statements (
    id             bigserial   PRIMARY KEY,
    file_sha256    text        NOT NULL CHECK (file_sha256 ~ '^[0-9a-f]{64}$'),
    file_name      text        NOT NULL,
    format         text        CHECK (format IN ('pdf', 'csv', 'xls', 'xlsx')),
    kind           text        CHECK (kind IN ('statement', 'payment_history')),
    status         text        NOT NULL DEFAULT 'unconfirmed' CHECK (status IN ('unconfirmed', 'confirmed')),
    outcome        text        NOT NULL CHECK (outcome IN ('read', 'locked', 'unsupported', 'no_table', 'crashed')),
    result         jsonb,
    uploaded_at    timestamptz NOT NULL DEFAULT now(),
    confirmed_at   timestamptz,
    read_at        timestamptz,
    reader_version text,
    CHECK ((status = 'confirmed') = (confirmed_at IS NOT NULL))
);
CREATE INDEX statements_sha_idx ON finance.statements (file_sha256);

-- The Accounts a Statement lists, in printed order. `account_id` is set once the person
-- confirms (or picks a match); the proposed_* columns are what the reader found.
CREATE TABLE finance.statement_accounts (
    statement_id    bigint NOT NULL REFERENCES finance.statements (id) ON DELETE CASCADE,
    position        int    NOT NULL CHECK (position >= 0),
    account_id      bigint REFERENCES finance.accounts (id),
    proposed_issuer text,
    proposed_kind   text   CHECK (proposed_kind IN ('savings', 'credit_card', 'loan')),
    proposed_mask   text,
    PRIMARY KEY (statement_id, position)
);

CREATE TABLE finance.transactions (
    id                 bigserial   PRIMARY KEY,
    account_id         bigint      NOT NULL REFERENCES finance.accounts (id),
    statement_id       bigint      NOT NULL REFERENCES finance.statements (id),
    date               date        NOT NULL,
    description        text        NOT NULL,
    amount_paise       bigint      NOT NULL CHECK (amount_paise > 0),
    direction          text        NOT NULL CHECK (direction IN ('debit', 'credit')),
    balance_paise      bigint,
    foreign_amount     text,
    foreign_currency   text,
    reference          text,
    addon_card         text,
    source_id          text,
    fingerprint        text        NOT NULL,
    no_longer_found_at timestamptz,
    created_at         timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX transactions_fingerprint_idx ON finance.transactions (account_id, fingerprint);
CREATE INDEX transactions_date_idx ON finance.transactions (account_id, date);
CREATE INDEX transactions_statement_idx ON finance.transactions (statement_id);

-- What a payment app noted about one payment. Never money; links to at most one Transaction.
CREATE TABLE finance.payment_records (
    id             bigserial PRIMARY KEY,
    statement_id   bigint    NOT NULL REFERENCES finance.statements (id),
    occurred_at    timestamp NOT NULL,
    description    text      NOT NULL,
    amount_paise   bigint    NOT NULL CHECK (amount_paise > 0),
    direction      text      NOT NULL CHECK (direction IN ('debit', 'credit')),
    source_id      text,
    utr            text,
    instrument     text,
    transaction_id bigint    REFERENCES finance.transactions (id) ON DELETE SET NULL
);
CREATE UNIQUE INDEX payment_records_source_idx ON finance.payment_records (source_id) WHERE source_id IS NOT NULL;
-- A Transaction carries at most one Payment record.
CREATE UNIQUE INDEX payment_records_transaction_idx ON finance.payment_records (transaction_id) WHERE transaction_id IS NOT NULL;

-- Write-only: the API lists labels, never the secret.
CREATE TABLE finance.statement_passwords (
    id           bigserial   PRIMARY KEY,
    label        text        NOT NULL,
    secret       text        NOT NULL,
    created_at   timestamptz NOT NULL DEFAULT now(),
    last_used_at timestamptz
);

-- One row per read, confirm or Read again attempt, failed ones too.
CREATE TABLE finance.runs (
    id           bigserial   PRIMARY KEY,
    kind         text        NOT NULL CHECK (kind IN ('read', 'confirm', 'read_again')),
    started_at   timestamptz NOT NULL,
    finished_at  timestamptz NOT NULL,
    outcome      text        NOT NULL CHECK (outcome IN ('success', 'failed')),
    counts       jsonb       NOT NULL DEFAULT '{}',
    error_code   text,
    statement_id bigint
);
CREATE INDEX runs_kind_started_idx ON finance.runs (kind, started_at DESC);
