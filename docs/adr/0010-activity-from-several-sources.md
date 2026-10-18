> [ADR 0011](0011-a-stored-timeline-merged-per-trait.md) amends "combining happens only when Activity is read": combining now happens when the Timeline is built.

# Activity from several Sources

Activity comes from more than one tracking app: Toggl Track for Mac and Clockify Desktop both record the frontmost app and window, and each misses days the other has. We keep all of it in one table, `time.activities`, with a `source` column. Shared columns hold what every Source records; a few nullable columns hold what only one does (Toggl's device and idle flag, Clockify's page address and idle seconds), and a check keeps each Source's columns empty for the other. The primary key is `(source, source_id)`, so the same id under two Sources is two rows.

Every row is stored exactly as its Source recorded it. Combining Sources (for example, preferring Clockify's idle seconds, the only usable away signal, over Toggl's missing one) happens only when Activity is read, so a better rule later needs no re-Import and no data is lost to an early guess. Each Source has its own Snapshot and its own Import button, and `time.runs` names the Source of each Import. The helper takes one Snapshot per Source and one failing never blocks the other (ADR 0009).

## Considered Options

- **A separate table per Source.** Rejected: every reader would need a union, and a third Source would add another table and another copy of the same upsert.
- **Combining at Import time** (one merged row per stretch of time). Rejected: it throws away what each Source said and bakes in a precedence rule that is easier to change on read.
- **A generic key-value or JSON-only table.** Rejected: apps, times and ids are queried constantly, so they stay real, typed columns.

## Consequences

- The migration turns the existing Toggl rows into `source = 'toggl'` and renames `toggl_id` to `source_id` (text, the same lowercase UUID form).
- Adding a Source is one descriptor, one reader and parser, one helper job and one migration that widens the `source` checks.
- Analysis must say which Source it reads, or combine them deliberately.
