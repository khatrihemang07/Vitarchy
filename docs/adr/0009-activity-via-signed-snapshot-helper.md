# Activity via a signed snapshot helper

The Toggl Track for Mac app keeps Activity in a macOS-protected group container that only a process with Full Disk Access (FDA) can read. We chose a small standalone helper (`crates/helpers/activity-snapshot`), signed with a self-signed "Vitarchy Dev" identity and run by a launchd agent every 8 hours and at login. It copies the data with SQLite's online backup API, which captures the write-ahead log, into Vitarchy's own folder. The person grants FDA to that one binary. The server then Imports from the Snapshot and never touches the protected folder.

The helper must be started by launchd, not by the server. macOS attributes protected-file access to the "responsible process", which for a spawned child is the process that started it. A helper spawned by the server would run under the terminal or editor that launched the server, and would need that app to hold FDA.

## Considered Options

- **Borrow the terminal's or VS Code's FDA.** Rejected: it grants broad access to everything run from that app, and it breaks whenever the server starts from somewhere else.
- **Upload the file by hand each time.** Rejected: it is easy to forget, and Activity would silently stop growing.
- **Toggl's server-side timeline sync.** Rejected: it sends window titles off the machine, is undocumented and keeps only 90 days.
- **A Shortcuts automation.** Rejected: it cannot do a consistent SQLite backup, and its file access is just as restricted.

## Consequences

- Replacing the signing identity invalidates the FDA grant, so `make signing-setup` refuses to recreate it without `--force`.
- Reinstalling the helper under the same identity and path keeps the grant.
- A Snapshot is at most 8 hours old, and its file modification time is its age.
