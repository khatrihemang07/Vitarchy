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
_Avoid_: Tracking

**Timeline**:
The single account of what happened on this computer, built from every Source's Activity by drawing on what each Source records best. It never overlaps itself, so every moment is counted at most once. It is always derived from Activity and can be rebuilt from it at any time; it is never edited by hand. With one Source it follows that Source's Activity, and with none it is empty.
_Avoid_: Merged activity, combined activity

**Segment**:
One stretch of the Timeline in a single app and window, which notes the Sources that back it. Each Segment is Away, not Away, or of unknown Away when no Source that covers it can tell.
_Avoid_: Block, slice, span

**No input**:
A stretch with no keyboard or mouse use, as a Source reports it. On its own it is not Away: a pause to read or think is no input too.
_Avoid_: Idle, inactive

**Away**:
Time when the person was not using the computer, even though a Source kept recording the frontmost app. Only a No input stretch of an hour or more counts as Away, starting at the last input; shorter pauses stay Active. While watching or in a call, up to 2 hours of No input stays Active. Only Sources that notice inactivity can say a stretch was Away; where none does, Away is unknown, not assumed.
_Avoid_: Idle, inactive, AFK

**Active**:
Timeline time known not to be Away. Time whose Away is unknown is never counted as Active.
_Avoid_: Busy, working time, productive time

**Switch**:
Moving from one app to a different app between two touching Active Segments. A new window within the same app is not a Switch, and Away, unknown Away or a gap in between means there is no Switch.
_Avoid_: Context switch, toggle, jump

**Focus stretch**:
Continuous Active time in one app, carrying on through interruptions shorter than a minute and ended by Away or unknown Away. Stretches of 25 minutes or more count as focused time.
_Avoid_: Session, streak, deep work

**Source**:
A tracking app whose local data Vitarchy Imports Activity from, such as Toggl Track or Clockify Desktop. Each Source's Activity is kept as that Source recorded it; Activity from several Sources is combined only when it is read. Toggl Track plays two separate roles: the Mac app is a Source of Activity, and the Toggl connection is where Time Entries are Synced from. The two never mix.
_Avoid_: Tracker, provider, integration (Source is a Time word; Finance's files are Statements)

**Sync**:
Bringing the person's Time Entries from Toggl into Vitarchy so the copy matches Toggl, including deletions. The person starts it with a button.
_Avoid_: Refresh, pull, update

**Import**:
Bringing Activity from one Source's latest Snapshot into Vitarchy. It adds new Activity and updates Activity it already holds (such as one that has grown longer), but never removes any, so Vitarchy keeps Activity that the Source later discards. The person starts it with a button.
_Avoid_: Sync, load

**Snapshot**:
A consistent copy of one Source's local data, taken on a schedule, from which Activity is Imported. Its age is when it was taken.
_Avoid_: Backup, export, dump

**Toggl connection**:
The person's link between Vitarchy and their Toggl Track account, made by entering their Toggl API token. The token is never shown again.
_Avoid_: Integration, login, account

## Analysing time

**Day**:
One calendar day in the person's own time zone, from midnight to midnight. A Time Entry that spans midnight counts toward each Day it touches, only for the part inside that Day.
_Avoid_: Date, 24 hours

**Period**:
The stretch of Days a Report covers: one Day, a week, a month, or a custom stretch of Days. Weeks start on the Core week-start Setting.
_Avoid_: Timeframe, window

**Report**:
A summary of the Time Entries in one Period, filtered and grouped the way the person chooses, and compared with either the Previous Period or the Usual.
_Avoid_: Analysis, dashboard, insights

**Previous Period**:
The closest earlier Period of the same kind that has Time Entries matching the filters. Usually yesterday, last week, last month, or the same-length stretch before a custom one; after a gap, the last one that had entries.
_Avoid_: Last period, baseline

**Usual**:
How a group usually went over the 4 weeks (28 Days) just before a Period: its per-Day average, minimum and maximum over the Days it happened, and how many Days that was. The Usual applies to Timeline figures in the same way, over the Days that have Timeline. When a Period is compared with the Usual, totals and counts are scaled to the Days that have passed in the Period, against the Days of history available.
_Avoid_: Baseline, norm, typical

## Finance

**Statement**:
A file from a bank or card issuer (PDF, CSV, XLS or XLSX) that lists the Transactions of one or more Accounts over a period. It is unconfirmed until the person checks its preview and confirms it; only then do its Transactions count. Vitarchy keeps the original file so it can be Read again.
_Avoid_: Source, feed, export, report

**Account**:
A savings account, credit card or loan at one issuer, known by its masked number. Each card on a bill is its own Account; a bill that lists several cards is one Statement with several Accounts.
_Avoid_: Wallet, card (as a general word)

**Transaction**:
One movement of money on one Account, exactly as its Statement printed it: the date, the description, the amount, whether it was a debit or a credit, and the balance after it when printed. The same Transaction appearing in two overlapping Statements is stored once.
_Avoid_: Entry, record, txn

**Payment record**:
What a payment app such as PhonePe noted about one payment: when, with whom, its ids, and the Account it used. It is never money on its own; it links to at most one Transaction, and it waits unlinked until that Account's Statement arrives.
_Avoid_: UPI record, payment detail

**Payment history**:
A payment app's export of Payment records. It is not a Statement and creates no Transactions.
_Avoid_: Statement (for app exports)

**Statement password**:
A password the person saved for opening locked Statements. Each one is tried on every locked file and never shown again.
_Avoid_: PIN, key

**Check**:
A test that a Statement's Transactions agree with each other and with the totals it prints: the running balance follows row to row, the debits and credits sum to the printed totals, and the counts match. A failed Check is a warning, not a rejection, because issuers sometimes print inconsistent figures.
_Avoid_: Validation, reconciliation

**Read again**:
Running the current reading engine over a stored Statement, to find Transactions an earlier engine missed. It adds what is new and marks what it no longer finds, but never deletes.
_Avoid_: Re-import, reparse, refresh
