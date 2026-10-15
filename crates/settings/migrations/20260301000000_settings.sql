CREATE SCHEMA IF NOT EXISTS settings;

-- One JSON document per scope: 'core', or a Module id.
CREATE TABLE settings.entries (
    scope      text        PRIMARY KEY,
    value      jsonb       NOT NULL,
    updated_at timestamptz NOT NULL DEFAULT now()
);
