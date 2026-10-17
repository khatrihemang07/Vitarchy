-- Numbered 20260407: versions 20260404-20260406 belonged to migrations removed in 065bbb8
-- that are still recorded in live databases, so they must never be reused.
-- Activity from several Sources (Toggl, Clockify) in one table. Each row is stored as
-- its Source recorded it; combining happens only on read (ADR 0010).
ALTER TABLE time.activities
    ADD COLUMN source text NOT NULL DEFAULT 'toggl' CHECK (source IN ('toggl', 'clockify'));
-- Existing rows are all Toggl's; later inserts must name their Source.
ALTER TABLE time.activities ALTER COLUMN source DROP DEFAULT;

ALTER TABLE time.activities DROP CONSTRAINT activities_pkey;
ALTER TABLE time.activities RENAME COLUMN toggl_id TO source_id;
-- uuid::text is the lowercase hyphenated form the Toggl parser writes.
ALTER TABLE time.activities ALTER COLUMN source_id TYPE text USING source_id::text;
ALTER TABLE time.activities ADD PRIMARY KEY (source, source_id);

-- Source-specific extras. Toggl flags idle; Clockify reports idle seconds and page addresses.
ALTER TABLE time.activities ALTER COLUMN idle DROP NOT NULL;
ALTER TABLE time.activities ADD COLUMN url text;
-- Idle seconds as Clockify reports them; can exceed the span (kept as-is).
ALTER TABLE time.activities ADD COLUMN idle_secs double precision;
ALTER TABLE time.activities ADD CONSTRAINT activities_source_columns CHECK (
    CASE source
        WHEN 'toggl'    THEN idle IS NOT NULL AND url IS NULL AND idle_secs IS NULL
        WHEN 'clockify' THEN idle IS NULL AND client_id IS NULL AND idle_secs IS NOT NULL
    END
);

-- An Import run names its Source; a Sync run has none.
ALTER TABLE time.runs ADD COLUMN source text CHECK (source IN ('toggl', 'clockify'));
UPDATE time.runs SET source = 'toggl' WHERE kind = 'import';
ALTER TABLE time.runs ADD CONSTRAINT runs_source_matches_kind
    CHECK ((kind = 'import') = (source IS NOT NULL));
