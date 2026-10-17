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
What a tracking app passively noticed on this computer: which app and window was in use, and when. Each piece of Activity comes from exactly one Source. It is separate from Time Entries and is analysed separately.
_Avoid_: Timeline, tracking

**Source**:
A tracking app whose local data Vitarchy Imports Activity from, such as Toggl Track or Clockify Desktop. Each Source's Activity is kept as that Source recorded it; Activity from several Sources is combined only when it is read.
_Avoid_: Tracker, provider, integration

**Sync**:
Bringing the person's Time Entries from Toggl into Vitarchy so the copy matches Toggl, including deletions. The person starts it with a button.
_Avoid_: Refresh, pull, update

**Import**:
Adding Activity from one Source's latest Snapshot to Vitarchy. It only adds, so Vitarchy keeps Activity that the Source later discards. The person starts it with a button.
_Avoid_: Sync, load

**Snapshot**:
A consistent copy of one Source's local data, taken on a schedule, from which Activity is Imported. Its age is when it was taken.
_Avoid_: Backup, export, dump

**Toggl connection**:
The person's link between Vitarchy and their Toggl Track account, made by entering their Toggl API token. The token is never shown again.
_Avoid_: Integration, login, account
