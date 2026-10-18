# Statement originals kept on disk and Read again

Every uploaded Statement is kept as a decrypted file named by its SHA-256, in `FINANCE_STATEMENTS_DIR` (live: `~/Library/Application Support/Vitarchy/statements/`; stage and tests use their own folder). The database holds the hash. Keeping the original lets a better engine Read it again without the person finding and unlocking the file a second time. This follows ADR 0010's rule: store what the source said, derive later.

Read again never deletes. It adds Transactions it newly finds and marks the ones it no longer finds, after showing a preview. A weaker engine run can therefore never lose confirmed data.

## Considered Options

- **Bytes in Postgres.** This would separate live and stage automatically and be included in backups. The person chose a folder on disk instead, so `make backup` and `make restore` archive and restore the folder next to the SQL dump.
- **Keep the file still encrypted.** Rejected: Read again would need the password every time.
- **Discard after reading.** Rejected: an engine fix could not reach old Statements.

## Consequences

- The folder holds decrypted personal statements. Treat it, and the backups that include it, as secret.
- A restore without its folder leaves Statements that cannot be Read again, but their Transactions are intact.
