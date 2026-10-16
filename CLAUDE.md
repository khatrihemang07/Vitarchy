## Code quality: always

Before writing, changing or reviewing code, or briefing a subagent to, read `CODING_STANDARDS.md` and follow it. Include it in every subagent brief. The non-negotiables:

- **Deep, single-purpose modules**: one reason to change per file, one component per file, pure logic in tested functions.
- **Single source of truth**: derive values; never restate them.
- **Boundaries**: code lives where `CODING_STANDARDS.md` § Where code lives says. Modules never import each other.
- **Debuggable**: structured logs at boundaries with the request id. Never log secrets or entered values.
- **Real data is sacred**: the live database (`:5433`, `.env`) holds the user's real data. Tests and every agent or manual check use **stage** (`make stage`, `make smoke`, `make test`, `TEST_DATABASE_URL` from `.env.stage`), never live. Leave the user's running servers untouched.
- **Done** means its Definition of done checklist is ticked.

## Agent skills

### Issue tracker

Issues live in GitHub Issues for this repo (via the `gh` CLI). See `docs/agents/issue-tracker.md`.

### Triage labels

Default five-label vocabulary (`needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`). See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: one `GLOSSARY.md` + `docs/adr/` at the repo root. See `docs/agents/domain.md`.
