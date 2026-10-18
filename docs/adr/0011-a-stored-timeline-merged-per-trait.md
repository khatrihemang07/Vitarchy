# A stored Timeline, merged per trait

Each Source's Activity is stored raw ([ADR 0010](0010-activity-from-several-sources.md)), but nobody wants to read two overlapping accounts of the same computer. The Timeline is the single, non-overlapping account built from all of them: a stored table of Segments, derived from Activity and rebuildable from it at any time. Raw Activity is never changed; the Timeline is never edited by hand.

Real Snapshots showed why a plain "pick one Source" rule is not enough. Both apps agree on the app for almost every shared minute, yet each misses whole days the other has. Neither stops recording when the person is away: a single Toggl row can run for days. Toggl's idle flag is never set, so Toggl cannot say when the person was Away. Clockify's `idle_secs` is the only away signal, and it is cumulative (it can exceed the row) and its rows split at midnight. Toggl has finer window-title boundaries; Clockify has page addresses.

So each Source declares its strengths per trait (`boundaries`, which also decides app and title, `url`, and `away`) as data on its descriptor, together with an `observe` function that normalises its own rows. For each moment the engine takes each trait from the highest-ranked Source that covers it and has that trait, so it handles 0, 1 or N Sources with the same code: no Source gives an empty Timeline, and one Source gives that Source's Activity. A Segment lists every Source that covered it.

Two rules follow from the data. Clockify's idle is the trailing part of its row (the person stopped, then the row ended), capped at the row's length, so a row becomes an active part followed by an Away part. And Away is never guessed: where no covering Source can tell (for example Toggl alone), the Segment's Away is unknown. Segments are stored in UTC and not split at midnight, because the time zone arrives per request and Reports split into Days on read. Each Import rebuilds only the range it touched, widened to the Activity that overlaps it, in one transaction; the person can also rebuild everything with a button.

## Considered Options

- **Compute the merge on every read.** Rejected: every reader would pay for the sweep over all Activity and would have to agree on the rules, and a slow merge would sit in front of every Report.
- **A fixed Source precedence** (for example Clockify always wins). Rejected: no Source is best at everything. Toggl has the finer boundaries and the longer history; Clockify alone knows the URL and Away. Per-trait strengths keep each Source's best part, and a new Source is data, not a new branch.
- **Merging at Import time into Activity.** Rejected: it would throw away what each Source said and bake in a rule that cannot be improved without re-Importing, which is impossible once a Source has discarded old data.

## Consequences

- ADR 0010's rule "combining happens only when Activity is read" is amended: combining now happens when the Timeline is built, and Activity itself stays untouched.
- A better merge rule later is a rebuild, not a re-Import and not a data loss.
- Adding a Source means declaring its `strengths` and `observe` on its descriptor as well as its reader.
- The Timeline is derived storage: it can be deleted and rebuilt, and a failed rebuild fails the Import run that triggered it, so the two never silently disagree.
- Away stays unknown for stretches only Toggl covers; the UI says so rather than showing a guess.
