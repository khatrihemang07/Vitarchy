-- Fake Time data for STAGE only (run by scripts/stage-seed.sh, i.e. `make stage-seed`).
-- Everything it writes has workspace_id 9000001, so a re-run first deletes its own rows
-- and leaves anything else alone. Dates are relative to now(), in UTC.
\set ON_ERROR_STOP on
BEGIN;

DO $$ BEGIN
  IF to_regclass('time.time_entries') IS NULL THEN
    RAISE EXCEPTION 'the time schema is missing: start the stage server once (make stage) so it migrates';
  END IF;
END $$;

DELETE FROM time.time_entries WHERE workspace_id = 9000001;
DELETE FROM time.projects     WHERE workspace_id = 9000001;
DELETE FROM time.clients      WHERE workspace_id = 9000001;
DELETE FROM time.tags         WHERE workspace_id = 9000001;

-- Connected, so the Time page shows its connected state. The token is not a real one.
INSERT INTO time.connection (api_token, workspace_id, workspace_name, toggl_user_id)
VALUES ('stage-seed-not-a-real-token', 9000001, 'Stage workspace', 9000001)
ON CONFLICT (singleton) DO NOTHING;
INSERT INTO time.mirror_owner (toggl_user_id, workspace_id)
VALUES (9000001, 9000001)
ON CONFLICT (singleton) DO NOTHING;

INSERT INTO time.clients (toggl_id, workspace_id, name, archived, raw) VALUES
  (9200001, 9000001, 'Acme Corp',    false, '{}'::jsonb),
  (9200002, 9000001, 'Freelance Co', false, '{}'::jsonb);

INSERT INTO time.projects (toggl_id, workspace_id, client_id, name, active, color, raw) VALUES
  (9100001, 9000001, 9200001, 'Office',       true, '#06aaf5', '{}'::jsonb),
  (9100002, 9000001, NULL,    'Gym',          true, '#2da608', '{}'::jsonb),
  (9100003, 9000001, 9200002, 'Side project', true, '#c9806b', '{}'::jsonb);

INSERT INTO time.tags (toggl_id, workspace_id, name, raw) VALUES
  (9300001, 9000001, 'focus',   '{}'::jsonb),
  (9300002, 9000001, 'meeting', '{}'::jsonb),
  (9300003, 9000001, 'commute', '{}'::jsonb);

-- One row per planned entry: n days ago, a slot (keeps ids unique), start time, hours,
-- project, description and tags. The mixed spellings ("office ") are on purpose.
CREATE TEMP TABLE planned ON COMMIT DROP AS
WITH days AS (
  SELECT n, (now() AT TIME ZONE 'utc')::date - n AS day FROM generate_series(1, 60) AS n
)
-- Weekday mornings at the Office, spelled three ways.
SELECT n, 1 AS slot, day + time '08:00' AS start_at, 4.0 AS hours, 9100001 AS project,
       (ARRAY['Office', 'office ', ' Office'])[1 + n % 3] AS description,
       CASE WHEN n % 2 = 0 THEN ARRAY['focus'] ELSE ARRAY[]::text[] END AS tags
FROM days WHERE extract(isodow FROM day) < 6
UNION ALL -- Every fourth weekday a second Office entry: two entries, same description, one Day.
SELECT n, 2, day + time '13:00', 3.5, 9100001, 'Office', ARRAY['focus', 'meeting']
FROM days WHERE extract(isodow FROM day) < 6 AND n % 4 = 0
UNION ALL
SELECT n, 3, day + time '07:15', 0.75, 9100001, 'Commute to office', ARRAY['commute']
FROM days WHERE extract(isodow FROM day) < 6 AND n % 3 = 1
UNION ALL
SELECT n, 4, day + time '18:00', 1.0, 9100002, 'Gym', ARRAY[]::text[]
FROM days WHERE n % 4 = 0
UNION ALL
SELECT n, 5, day + time '10:00', 3.0, 9100003, 'Side project', ARRAY['focus']
FROM days WHERE extract(isodow FROM day) >= 6
UNION ALL -- Crosses midnight, so it counts toward two Days.
SELECT 5, 6, (now() AT TIME ZONE 'utc')::date - 5 + time '23:00', 2.5, 9100003, 'Late deploy',
       ARRAY['focus'];

INSERT INTO time.time_entries
  (toggl_id, workspace_id, project_id, description, tags, start, stop, duration_s, raw)
SELECT 9000000000 + n * 10 + slot, 9000001, project, description, tags,
       start_at AT TIME ZONE 'utc',
       (start_at + make_interval(secs => hours * 3600)) AT TIME ZONE 'utc',
       (hours * 3600)::bigint, '{}'::jsonb
FROM planned;

COMMIT;
SELECT count(*) AS seeded_time_entries FROM time.time_entries WHERE workspace_id = 9000001;
