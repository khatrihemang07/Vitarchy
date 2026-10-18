# Statements read by a Python helper spawned per file

Statements arrive as PDFs, CSVs and spreadsheets from many issuers, each with its own layout: borderless tables, side panels beside the table, narration wrapped over several lines, headers drawn as vector shapes, and several Accounts in one file. Reading a sample of 31 real files showed that table detectors (pymupdf `find_tables`, pdfplumber, docling) miss most of these. Word coordinates read with pymupdf, plus a header-anchored column layout and date-anchored rows, recovered every row in every file in about 20 ms per page.

We read Statements with `crates/helpers/statement-reader`, a Python program managed with uv. The Finance server spawns it once per file. The request (file path, Statement passwords) goes in on stdin, and a versioned JSON result (Transactions, Account, period, Checks) comes back on stdout, as described in its `CONTRACT.md`. The helper does the whole reading pipeline, driven by a shared vocabulary file, with no code for any one bank. Rust only stores, removes duplicates and links.

## Considered Options

- **Pure Rust** (pdfium or mupdf bindings). Rejected: there is no mature word-geometry and table tooling, and we would rebuild what pymupdf gives for free.
- **A long-running Python sidecar.** Rejected: another process to start and supervise, and it is only worth it for slow models (docling) that we don't use.
- **docling as the reader.** Rejected: 18–93 s per file, it missed SBI Card's table, and its columns slip. The `PageReader` seam lets an OCR or docling reader be added later for scanned files.
- **Splitting the work** (Python returns raw cells, Rust maps and normalises). Rejected: column mapping, normalising and Checks would then be spread over two languages.

## Consequences

- The server needs the helper installed (`make finance-reader-install`). Its path comes from `FINANCE_READER`, and tests use a fake reader.
- Unlike `activity-snapshot` (ADR 0009), the server starts this helper itself. That is fine because it reads only the uploaded file and needs no Full Disk Access.
- The contract is versioned. Both sides check it against shared fixtures.
