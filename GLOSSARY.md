# Vitarchy

A personal life-management app, built up from self-contained areas of life (time, money, and so on) that are added one at a time.

## Language

**Module**:
A self-contained area of life management, such as Time or Finance, that owns its records and its screens.
_Avoid_: Plugin, feature, app, domain

**Core**:
The shared ground that Modules may rely on. It holds the Settings that apply across all of Vitarchy, and no other domain concepts.
_Avoid_: Common, shared, base, kernel

**Live**:
The environment holding the person's real data, used day to day (`make start`). Never used for tests or automated checks.
_Avoid_: Prod, production, dev database

**Stage**:
The throwaway environment, with its own RAM-only database, where all tests and agent checks run (`make test`, `make smoke`, `make stage`).
_Avoid_: Test database, staging

**Setting**:
A value the person chooses that changes how Vitarchy looks or behaves, such as the theme or the first day of the week. Every Setting has a default and belongs to exactly one scope: Core or a single Module.
_Avoid_: Preference, option, config

**Core settings**:
The Settings that apply across all of Vitarchy: display name, theme, motion, week start, clock and date format.
_Avoid_: Common settings, global settings, general settings

**Module settings**:
The Settings that belong to one Module and only change that Module.
_Avoid_: Plugin settings, feature flags

**Time Entry**:
A span of time the person deliberately recorded in Toggl Track, usually with a description and a project. Toggl owns it, and Vitarchy keeps a read-only copy.
_Avoid_: Log, record

**Activity**:
What the Toggl Track app passively noticed on this computer: which app and window was in use, and when. It is separate from Time Entries and is analysed separately.
_Avoid_: Timeline, tracking

**Sync**:
Bringing the person's Time Entries from Toggl into Vitarchy so the copy matches Toggl, including deletions. The person starts it with a button.
_Avoid_: Refresh, pull, update

**Import**:
Adding Activity from the latest Snapshot to Vitarchy. It only adds, so Vitarchy keeps Activity that Toggl later discards. The person starts it with a button.
_Avoid_: Sync, load

**Snapshot**:
A consistent copy of the Toggl Track app's local data, taken on a schedule, from which Activity is Imported. Its age is when it was taken.
_Avoid_: Backup, export, dump

**Toggl connection**:
The person's link between Vitarchy and their Toggl Track account, made by entering their Toggl API token. The token is never shown again.
_Avoid_: Integration, login, account

## Analysing time

**Day**:
One calendar day in the person's own time zone, from midnight to midnight. Anything that spans midnight counts toward each Day it touches, only for the part inside that Day.
_Avoid_: Date, 24 hours

**Period**:
The stretch of Days the person is looking at: one Day, a week, a month, or a custom range. Weeks start on the Core week-start Setting.
_Avoid_: Range, timeframe, window

**Unlogged**:
The part of a Day that no Time Entry covers. Overlapping entries are not counted twice, and the part of today that has not happened yet is not Unlogged.
_Avoid_: Idle, free time, gap

**Context**:
What a piece of Activity was spent on within an app: the code project in an editor, the site in a browser, otherwise the app itself.
_Avoid_: Category, tag, project

**Routine**:
An ordered list of Steps the person goes through in a Day, such as the trip to the office, the time there, and the trip home. The person confirms or renames a Routine that Vitarchy suggests.
_Avoid_: Habit, schedule, workflow

**Step**:
One part of a Routine, matched to a Time Entry description or project.
_Avoid_: Stage, task

**Occurrence**:
One Day on which a Routine happened. It is complete when every Step happened, and incomplete when some are missing.
_Avoid_: Instance, run

**Insight**:
A plain statement the person can act on, found by simple statistics on their own Time Entries and Activity, such as a typical time, a typical duration, or a change from before.
_Avoid_: Tip, suggestion, recommendation
