-- Routines: ordered Steps the person goes through in a Day. Names are unique ignoring case.
CREATE TABLE time.routines (
    id         bigserial   PRIMARY KEY,
    name       text        NOT NULL CHECK (btrim(name) <> '' AND char_length(name) <= 80),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX routines_name_idx ON time.routines (lower(name));

-- A Step matches Time Entries by description or by project name (see analysis::routines::Step).
CREATE TABLE time.routine_steps (
    routine_id  bigint NOT NULL REFERENCES time.routines (id) ON DELETE CASCADE,
    position    int    NOT NULL CHECK (position >= 0),
    label       text   NOT NULL CHECK (btrim(label) <> ''),
    match_kind  text   NOT NULL CHECK (match_kind IN ('description', 'project')),
    match_value text   NOT NULL CHECK (btrim(match_value) <> ''),
    PRIMARY KEY (routine_id, position)
);
