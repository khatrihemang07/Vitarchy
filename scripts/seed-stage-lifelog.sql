-- SYNTHETIC life-log Time Entries for STAGE visual checks. Run by scripts/seed-stage-lifelog.sh,
-- which refuses anything but stage. Nothing here is copied from real data.
-- Variable: :tz (IANA zone the invented days are lived in).
\set ON_ERROR_STOP on
BEGIN;

-- Never clobber a connection that someone made by hand on stage.
DO $$
BEGIN
  IF EXISTS (SELECT 1 FROM time.connection WHERE workspace_id <> 900001) THEN
    RAISE EXCEPTION 'stage has a real Toggl connection; not seeding over it';
  END IF;
END $$;

-- Deterministic "randomness": the same day and salt always give the same number in 0..n-1.
CREATE FUNCTION pg_temp.jit(d date, salt text, n int) RETURNS int
  LANGUAGE sql IMMUTABLE AS $$ SELECT abs(hashtext(d::text || salt)) % n $$;
-- Minute `m` after local midnight of `d`, as an instant.
CREATE FUNCTION pg_temp.at(d date, m int, tz text) RETURNS timestamptz
  LANGUAGE sql IMMUTABLE AS $$ SELECT (d::timestamp + make_interval(mins => m)) AT TIME ZONE tz $$;

-- A fake connection, so the Time page shows a connected workspace.
INSERT INTO time.connection (api_token, workspace_id, workspace_name, toggl_user_id)
VALUES ('synthetic-stage-token', 900001, 'Synthetic workspace', 900001)
ON CONFLICT (singleton) DO NOTHING;
INSERT INTO time.mirror_owner (toggl_user_id, workspace_id) VALUES (900001, 900001)
ON CONFLICT (singleton) DO NOTHING;

INSERT INTO time.projects (toggl_id, workspace_id, name, active, color, raw) VALUES
  (9000001, 900001, 'office',    true, '#2a78d6', '{}'),
  (9000002, 900001, 'sleep',     true, '#6250d6', '{}'),
  (9000003, 900001, 'body',      true, '#1baf7a', '{}'),
  (9000004, 900001, 'essential', true, '#eda100', '{}'),
  (9000005, 900001, 'work',      true, '#e87ba4', '{}')
ON CONFLICT (toggl_id) DO UPDATE SET name = EXCLUDED.name, color = EXCLUDED.color;

DELETE FROM time.time_entries WHERE toggl_id >= 9000000000;

WITH days AS (
  SELECT g::date AS d, extract(isodow FROM g) < 6 AS weekday
  FROM generate_series(
         (now() AT TIME ZONE :'tz')::date - 59,
         (now() AT TIME ZONE :'tz')::date - 1,
         interval '1 day') AS g
),
-- Weekdays go to the office, except about one in twelve (leave).
trip AS (
  SELECT d,
         525 + pg_temp.jit(d, 'ts', 31)                     AS out_start,
         25 + pg_temp.jit(d, 'td', 11)                      AS out_len,
         420 + pg_temp.jit(d, 'od', 121)                    AS stay_len,
         30 + pg_temp.jit(d, 'fd', 16)                      AS back_len
  FROM days
  WHERE weekday AND pg_temp.jit(d, 'off', 12) <> 0
),
office AS (
  SELECT d, out_start, out_len,
         out_start + out_len + 2                   AS stay_start, stay_len,
         out_start + out_len + 2 + stay_len + 2    AS back_start, back_len
  FROM trip
),
spans (d, start_m, len_m, project_id, description) AS (
  SELECT d, out_start,  out_len,  9000001, 'travel to_office'   FROM office
  UNION ALL SELECT d, stay_start, stay_len, 9000001, 'inside_office'     FROM office
  UNION ALL SELECT d, back_start, back_len, 9000001, 'travel from_office' FROM office
  -- Sleep begins this evening and runs past midnight.
  UNION ALL SELECT d, 1390 + pg_temp.jit(d, 'sl', 60), 390 + pg_temp.jit(d, 'sd', 91), 9000002, 'sleep' FROM days
  UNION ALL SELECT d, 1185 + pg_temp.jit(d, 'wo', 16), 40 + pg_temp.jit(d, 'wd', 16), 9000003, 'workout'
         FROM days WHERE pg_temp.jit(d, 'w', 100) < 40
  UNION ALL SELECT d, 1265 + pg_temp.jit(d, 'dn', 21), 30 + pg_temp.jit(d, 'dd', 16), 9000004, 'dinner' FROM days
  UNION ALL SELECT d, 1335 + pg_temp.jit(d, 'bt', 16), 15 + pg_temp.jit(d, 'bd', 11), 9000004, 'bath' FROM days
  UNION ALL SELECT d, 660, 180, 9000005, 'side project'
         FROM days WHERE NOT weekday AND pg_temp.jit(d, 'sp', 2) = 0
)
INSERT INTO time.time_entries
  (toggl_id, workspace_id, project_id, description, tags, billable, start, stop, duration_s, raw)
SELECT 9000000000 + row_number() OVER (ORDER BY pg_temp.at(d, start_m, :'tz')),
       900001, project_id, description, '{}', false,
       pg_temp.at(d, start_m, :'tz'),
       pg_temp.at(d, start_m + len_m, :'tz'),
       len_m * 60, '{}'
FROM spans;

COMMIT;

SELECT count(*) AS synthetic_entries, min(start)::date AS first_day, max(start)::date AS last_day
FROM time.time_entries WHERE toggl_id >= 9000000000;
