-- The Away rules are fingerprinted from their values, not numbered by hand: the stored
-- fingerprint is compared with the one the code computes. The empty default differs from any
-- real fingerprint, so the Timeline is rebuilt once after this migration.
ALTER TABLE time.timeline_meta DROP COLUMN rules_version;
ALTER TABLE time.timeline_meta ADD COLUMN rules_fingerprint text NOT NULL DEFAULT '';
ALTER TABLE time.timeline_meta ALTER COLUMN rules_fingerprint DROP DEFAULT;
