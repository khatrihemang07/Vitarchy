-- The Timeline: one non-overlapping account of Activity from every Source, built per trait
-- from whichever Source records it best (ADR 0011). Always derived from time.activities, so
-- it can be dropped and rebuilt at any time; nothing here is the only copy of anything.
-- Stored in UTC and not split at midnight; Reports cut it into Days when they read it.
CREATE TABLE time.timeline_segments (
    start_at timestamptz PRIMARY KEY,
    end_at   timestamptz NOT NULL,
    app      text        NOT NULL,
    title    text        NOT NULL,
    url      text,
    -- 'unknown' when no Source covering the Segment can tell; never guessed.
    away     text        NOT NULL CHECK (away IN ('yes', 'no', 'unknown')),
    -- Every Source that covered it (their ids), in the Sources' listing order.
    sources  text[]      NOT NULL,
    built_at timestamptz NOT NULL DEFAULT now(),
    CHECK (end_at > start_at)
);
-- Segments never overlap, so the primary key orders them; this finds what ends after a time.
CREATE INDEX timeline_segments_end_idx ON time.timeline_segments (end_at);
