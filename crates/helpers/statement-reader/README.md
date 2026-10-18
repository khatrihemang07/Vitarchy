# statement-reader

Reads a bank, card or payment-app file (PDF, CSV, XLS, XLSX) into Transactions. The Finance server
spawns it once per uploaded file; the two talk only through [CONTRACT.md](CONTRACT.md) (version 1).
Why Python and why spawned per file: [ADR 0012](../../../docs/adr/0012-statements-read-by-a-python-helper.md).

## Install

Needs [uv](https://docs.astral.sh/uv/). From the repo root: `make finance-reader-install`
(or `uv sync` in this folder). Dependencies are pinned in `pyproject.toml` and `uv.lock`.

## Run

```
echo '{"version":1,"file_path":"/abs/path/file.pdf","passwords":["saved"],"decrypted_out":"/tmp/out"}' \
  | uv run statement-reader
```

The Result JSON goes to stdout and the exit code is 0 unless the helper crashes. Logs go to stderr as
JSON lines (stage names, counts and durations only; set `STATEMENT_READER_REQUEST_ID` to tag them).

## Layout

- `statement_reader/open/`: `sniff` (format from magic bytes) and `unlock` (passwords and case variants).
- `statement_reader/read/`: `pdf_words` (the `PageReader` seam: words and ruling lines) and `sheet` (a `Grid`).
- `statement_reader/vocabulary.toml`: header synonyms, date formats, stop markers, issuer markers. Data only.
- `statement_reader/locate/`: `header` (vocabulary match, stacked labels), `headerless` (columns from value types), `stops`, `regions`, `span`, `grid_header`.
- `statement_reader/segment/`: `anchors`, `cells`, `attach` (nearest-anchor narration), `rows`, `grid_rows`, `sections`, `wrap`.
- `statement_reader/normalize/`: `dates`, `amounts`, `direction`, `references`, `rows`, `payments`.
- `statement_reader/metadata/` (labelled facts, issuer, period) and `checks/figures.py` (balance chain, totals, counts).
- `statement_reader/engine.py`: `read_document(doc) -> Result`, composing `pdf_flow` and `sheet_flow`. No bank names in Python: a guard test enforces it.

## Test

- `uv run pytest` and `uv run ruff check .` (both run by `make test` and `make check`).
- Synthetic fixtures (one per hard layout trait, all invented) live in `tests/synthetic/files/`, with the
  Transactions planted in each under `expected/`. Regenerate with `uv run python -m tests.synthetic.make_fixtures`.
- `make finance-golden` also checks, per real file, hand-verified `description_samples` (kept in the private expected files) and that no description is empty or starts/ends with a bare amount; failures print only an index. It runs the helper over the real files in `fixtures/private/` (gitignored) and prints a
  pass/fail table of counts. `uv run python -m tests.golden.build_expected` rebuilds its expected files from
  `fixtures/private/analysis/`.
