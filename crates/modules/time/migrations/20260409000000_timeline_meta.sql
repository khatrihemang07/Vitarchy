-- What the stored Timeline was built under. One row, written after a rebuild: when the Away
-- rules in code (RULES_VERSION) differ from it, the Timeline is rebuilt once. No row means
-- it was built before versions existed, which also counts as different.
CREATE TABLE time.timeline_meta (
    only_row      boolean PRIMARY KEY DEFAULT true CHECK (only_row),
    rules_version integer NOT NULL
);
