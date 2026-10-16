-- Activity Imported from a Snapshot. Append-only: rows are never deleted, so
-- Vitarchy keeps Activity that Toggl later discards.
CREATE TABLE time.activities (
    toggl_id    uuid        PRIMARY KEY,
    client_id   uuid,
    app         text        NOT NULL,
    title       text        NOT NULL,
    start_at    timestamptz NOT NULL,
    end_at      timestamptz NOT NULL,
    idle        boolean     NOT NULL,
    -- Every source column, blobs as uuid strings, times as the original reals.
    raw         jsonb       NOT NULL,
    imported_at timestamptz NOT NULL DEFAULT now(),
    CHECK (end_at >= start_at)
);
CREATE INDEX activities_start_idx ON time.activities (start_at);
