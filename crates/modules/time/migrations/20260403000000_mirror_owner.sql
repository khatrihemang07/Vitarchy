-- Whose Toggl data the mirror holds. Unlike `time.connection` it survives a
-- disconnect, so connecting a different user or workspace afterwards still
-- clears the old person's Time Entries instead of mixing them in.
CREATE TABLE time.mirror_owner (
    singleton     boolean PRIMARY KEY DEFAULT true CHECK (singleton),
    toggl_user_id bigint  NOT NULL,
    workspace_id  bigint  NOT NULL
);
INSERT INTO time.mirror_owner (toggl_user_id, workspace_id)
SELECT toggl_user_id, workspace_id FROM time.connection;
