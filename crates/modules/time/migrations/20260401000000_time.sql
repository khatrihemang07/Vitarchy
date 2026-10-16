CREATE SCHEMA IF NOT EXISTS time;

-- The Toggl connection: at most one row. The token is write-only through the API.
CREATE TABLE time.connection (
    singleton      boolean     PRIMARY KEY DEFAULT true CHECK (singleton),
    api_token      text        NOT NULL,
    workspace_id   bigint      NOT NULL,
    workspace_name text        NOT NULL,
    toggl_user_id  bigint      NOT NULL,
    connected_at   timestamptz NOT NULL DEFAULT now(),
    -- Start time of the last successful Sync; the next Sync asks Toggl for changes since then.
    sync_cursor    timestamptz
);

-- Read-only mirror of Toggl Time Entries. Running entries are never stored.
CREATE TABLE time.time_entries (
    toggl_id         bigint      PRIMARY KEY,
    workspace_id     bigint      NOT NULL,
    project_id       bigint,
    task_id          bigint,
    description      text        NOT NULL DEFAULT '',
    tags             text[]      NOT NULL DEFAULT '{}',
    billable         boolean     NOT NULL DEFAULT false,
    start            timestamptz NOT NULL,
    stop             timestamptz NOT NULL,
    duration_s       bigint      NOT NULL,
    toggl_updated_at timestamptz,
    raw              jsonb       NOT NULL,
    synced_at        timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX time_entries_start_idx ON time.time_entries (start);

CREATE TABLE time.projects (
    toggl_id     bigint      PRIMARY KEY,
    workspace_id bigint      NOT NULL,
    client_id    bigint,
    name         text        NOT NULL,
    active       boolean     NOT NULL DEFAULT true,
    color        text,
    raw          jsonb       NOT NULL,
    synced_at    timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE time.clients (
    toggl_id     bigint      PRIMARY KEY,
    workspace_id bigint      NOT NULL,
    name         text        NOT NULL,
    archived     boolean     NOT NULL DEFAULT false,
    raw          jsonb       NOT NULL,
    synced_at    timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE time.tags (
    toggl_id     bigint      PRIMARY KEY,
    workspace_id bigint      NOT NULL,
    name         text        NOT NULL,
    raw          jsonb       NOT NULL,
    synced_at    timestamptz NOT NULL DEFAULT now()
);

-- One row per Sync or Import attempt, failed ones too; backs the status page.
CREATE TABLE time.runs (
    id          bigserial   PRIMARY KEY,
    kind        text        NOT NULL CHECK (kind IN ('sync', 'import')),
    started_at  timestamptz NOT NULL,
    finished_at timestamptz NOT NULL,
    outcome     text        NOT NULL CHECK (outcome IN ('success', 'failed')),
    counts      jsonb       NOT NULL DEFAULT '{}',
    error_code  text
);
CREATE INDEX runs_kind_started_idx ON time.runs (kind, started_at DESC);
