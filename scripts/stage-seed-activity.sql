-- Fake computer Activity for STAGE only (run by scripts/stage-seed.sh, i.e. `make stage-seed`).
-- About 5 weeks up to now, from both Sources, so the Timeline page has a Usual, Day, Week and
-- Month views to show. Every row's source id starts with '5eed' so a re-run deletes its own rows
-- and leaves anything else alone. Dates are relative to now(), in UTC. No real data: the apps are
-- common names and every title and address is invented (example.com).
--
-- What it covers on purpose:
--   * Clockify rows carry idle seconds, so their tails become Away;
--   * a Toggl-only stretch (days 12-17 ago, Away unknown) and a Clockify-only one (24-26 ago);
--   * no Activity at all on days 10, 21 and 32 ago, and a missing Zoom row every third day;
--   * rows crossing midnight (Toggl one row, Clockify split at midnight as it records them).
\set ON_ERROR_STOP on
BEGIN;

DO $$ BEGIN
  IF to_regclass('time.activities') IS NULL THEN
    RAISE EXCEPTION 'the time schema is missing: start the stage server once (make stage) so it migrates';
  END IF;
END $$;

DELETE FROM time.activities WHERE source_id LIKE '5eed%';

-- One working day: slot, start (minutes after midnight), length, app, title, address, idle share
-- (Clockify only). A '#' in a title becomes a number that varies by day. Lunch is the gap 12:30-13:30.
CREATE TEMP TABLE tpl (slot int, start_min int, dur_min int, app text, title text, url text, idle_pct int) ON COMMIT DROP;
INSERT INTO tpl VALUES
  (1,  510, 50, 'Visual Studio Code', 'report_#.rs — vitarchy',                 NULL,                                        0),
  (2,  560, 15, 'Safari',             'Documentation — Guide #',                'https://docs.example.com/guide/#',          0),
  (3,  575, 10, 'Slack',              'Team chat',                              NULL,                                        0),
  (4,  585, 75, 'Visual Studio Code', 'report_#.rs — vitarchy',                 NULL,                                        0),
  (5,  660, 20, 'Mail',               'Inbox (#)',                              NULL,                                       25),
  (6,  680, 25, 'Safari',             'Issue tracker — Ticket #',               'https://tracker.example.com/tickets/#',     0),
  (7,  720, 30, 'Terminal',           'cargo test — vitarchy',                  NULL,                                       60),
  (8,  810, 90, 'Visual Studio Code', 'timeline_#.rs — vitarchy',               NULL,                                        0),
  (9,  900, 45, 'Zoom',               'Weekly sync',                            NULL,                                        0),
  (10, 945, 40, 'Figma',              'Dashboard mockup #',                     'https://design.example.com/file/#',         0),
  (11, 985, 20, 'Notes',              'Meeting notes',                          NULL,                                       30),
  (12, 1005, 60, 'Visual Studio Code', 'timeline_#.rs — vitarchy',              NULL,                                        0);

-- The days with Activity: none 10, 21 and 32 days ago, and weekends only on every other day.
CREATE TEMP TABLE seed_days ON COMMIT DROP AS
SELECT n, (now() AT TIME ZONE 'utc')::date - n AS day
FROM generate_series(0, 35) AS n
WHERE n NOT IN (10, 21, 32)
  AND (extract(isodow FROM (now() AT TIME ZONE 'utc')::date - n) < 6 OR n % 2 = 0);

-- Every planned window as an instant range, cut off at now so today ends where the clock is.
CREATE TEMP TABLE seed_rows ON COMMIT DROP AS
SELECT d.n, t.slot, t.app, replace(t.title, '#', ((d.n + t.slot) % 4 + 1)::text) AS title,
       replace(t.url, '#', ((d.n + t.slot) % 4 + 1)::text) AS url, t.idle_pct,
       ((d.day + make_interval(mins => t.start_min)) AT TIME ZONE 'utc') AS start_at,
       least(((d.day + make_interval(mins => t.start_min + t.dur_min)) AT TIME ZONE 'utc'), now()) AS end_at
FROM seed_days d CROSS JOIN tpl t
WHERE NOT (t.slot = 9 AND d.n % 3 = 0)
  AND ((d.day + make_interval(mins => t.start_min)) AT TIME ZONE 'utc') < now();

-- Toggl: finer boundaries and window titles, no addresses, and it never says Away.
INSERT INTO time.activities (source, source_id, client_id, app, title, start_at, end_at, idle, raw)
SELECT 'toggl', '5eed0000-0000-4000-8000-' || lpad((n * 100 + slot)::text, 12, '0'), NULL,
       app, title, start_at, end_at, false, '{}'::jsonb
FROM seed_rows WHERE n NOT BETWEEN 24 AND 26;

-- Clockify: starts a little later, keeps the page address, and reports idle seconds (the Away tail).
INSERT INTO time.activities (source, source_id, app, title, url, start_at, end_at, idle_secs, raw)
SELECT 'clockify', '5eed-clockify-' || n || '-' || slot, app, title, url,
       least(start_at + make_interval(secs => (n % 3) * 30), end_at), end_at,
       round(extract(epoch FROM end_at - start_at) * idle_pct / 100.0), '{}'::jsonb
FROM seed_rows WHERE n NOT BETWEEN 12 AND 17;

-- A late session crossing midnight: one Toggl row; Clockify records it split at midnight.
INSERT INTO time.activities (source, source_id, client_id, app, title, start_at, end_at, idle, raw)
SELECT 'toggl', '5eed0000-0000-4000-8000-' || lpad('90003', 12, '0'), NULL, 'Visual Studio Code',
       'late_deploy.rs — vitarchy',
       (((now() AT TIME ZONE 'utc')::date - 3 + time '23:10') AT TIME ZONE 'utc'),
       (((now() AT TIME ZONE 'utc')::date - 2 + time '00:40') AT TIME ZONE 'utc'), false, '{}'::jsonb;
INSERT INTO time.activities (source, source_id, app, title, url, start_at, end_at, idle_secs, raw) VALUES
  ('clockify', '5eed-clockify-late-a', 'Visual Studio Code', 'late_deploy.rs — vitarchy', NULL,
   (((now() AT TIME ZONE 'utc')::date - 3 + time '23:10') AT TIME ZONE 'utc'),
   (((now() AT TIME ZONE 'utc')::date - 2) AT TIME ZONE 'utc'), 0, '{}'::jsonb),
  ('clockify', '5eed-clockify-late-b', 'Visual Studio Code', 'late_deploy.rs — vitarchy', NULL,
   (((now() AT TIME ZONE 'utc')::date - 2) AT TIME ZONE 'utc'),
   (((now() AT TIME ZONE 'utc')::date - 2 + time '00:40') AT TIME ZONE 'utc'), 600, '{}'::jsonb);

-- A Toggl-only row crossing midnight inside the Toggl-only stretch.
INSERT INTO time.activities (source, source_id, client_id, app, title, start_at, end_at, idle, raw)
SELECT 'toggl', '5eed0000-0000-4000-8000-' || lpad('90014', 12, '0'), NULL, 'Notes', 'Reading list',
       (((now() AT TIME ZONE 'utc')::date - 14 + time '22:30') AT TIME ZONE 'utc'),
       (((now() AT TIME ZONE 'utc')::date - 13 + time '01:00') AT TIME ZONE 'utc'), false, '{}'::jsonb;

COMMIT;
SELECT source, count(*) AS seeded_activity_rows FROM time.activities WHERE source_id LIKE '5eed%' GROUP BY source ORDER BY source;
