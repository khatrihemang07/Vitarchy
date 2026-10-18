# statement-reader contract, version 2

The Finance server runs `statement-reader` once per uploaded file. The two talk only through this contract. `statement_reader/contract.py` (Python) and `crates/modules/finance/src/reader/contract.rs` (Rust) both mirror it, and a contract test on each side checks that they still match `fixtures/contract/*.json`.

## Invocation

```
statement-reader < request.json > result.json
```

- **Exit 0:** a result JSON was written to stdout, whether or not the file could be read.
- **Non-zero exit:** a crash. The server treats it as `outcome: "crashed"`.
- **stderr:** structured log lines (JSON, one per line). They hold only stage names, counts and durations: never passwords, descriptions, amounts, names or account numbers. The server sets the `STATEMENT_READER_REQUEST_ID` environment variable to the id of the request that caused the read, and the helper tags each line with it, so a line can be matched to the server's log.

### Request (stdin)

```json
{
  "version": 2,
  "file_path": "/abs/path/to/upload",
  "passwords": ["…", "…"],
  "decrypted_out": "/abs/path/where/a/decrypted/copy/goes"
}
```

`passwords` are the saved Statement passwords, in order. The helper tries each one, together with its case variants (as-is, upper, lower, capitalised), opening a fresh handle for every attempt. When the file was encrypted and unlocks, the helper writes the decrypted bytes to `decrypted_out`. When it was not encrypted, the helper writes nothing there.

## Result (stdout)

```json
{
  "version": 2,
  "outcome": "read",
  "format": "pdf",
  "encrypted": true,
  "unlocked_with": 2,
  "kind": "statement",
  "statements": [Statement],
  "diagnostics": { "stage": null, "code": null, "pages": 4, "rows": 37 }
}
```

| field | values |
|---|---|
| `outcome` | `read` · `locked` (no password opened it) · `unsupported` (not a PDF/CSV/XLS/XLSX) · `no_table` (opened, but no transaction table found) |
| `format` | `pdf` · `csv` · `xls` · `xlsx` · `null` when unsupported |
| `unlocked_with` | index into `passwords` of the password whose variant worked, or `null` |
| `kind` | `statement` (bank or card) · `payment_history` (Payment history from a payment app) · `null` unless `outcome` is `read` |
| `statements` | `[]` unless `outcome` is `read`. For `statement`, one file may hold several (consolidated savings + loans, or a card bill listing several cards). For `payment_history`, one per distinct instrument (see below) |
| `diagnostics.stage` | when `outcome` is not `read`, the stage that stopped: `open` · `read` · `locate` · `segment` · `normalize` |
| `diagnostics.code` | a stable snake_case reason, e.g. `no_header_or_columns`, `no_rows` |

### Statement

```json
{
  "account": { "issuer": "ICICI Bank", "kind": "credit_card", "mask": "4123XXXXXXXX2003" },
  "period": { "start": "2026-06-19", "end": "2026-07-18" },
  "summary": {
    "opening_paise": 1234500, "closing_paise": 99900,
    "total_debit_paise": 500000, "total_credit_paise": 100000,
    "debit_count": 12, "credit_count": 3
  },
  "transactions": [Transaction],
  "checks": [Check]
}
```

- **`account`:**
  - Every field is nullable when the engine cannot tell.
  - `kind` is `savings`, `credit_card` or `loan`.
  - `mask` is the account or card number as printed, with at least the last 4 digits visible when printed.
- **`period`:** dates are ISO `YYYY-MM-DD`, and either can be `null`.
- **`summary`:** every figure is nullable and appears only when the Statement prints it. Paise are integers, signed for balances: a balance that is negative, or `DR` on a savings account, is negative.

### Transaction

```json
{
  "date": "2026-07-02",
  "time": null,
  "description": "UPI/DR/512345678901/DEMO GRO/HDFC/xyautopay./Indi",
  "amount_paise": 550000,
  "direction": "debit",
  "balance_paise": 8021778,
  "foreign_amount": "25.00",
  "foreign_currency": "USD",
  "reference": "512345678901",
  "source_id": null
}
```

- **`description`:** the text exactly as printed. Wrapped lines are joined: with a single space normally, and with nothing when the wrap split a token. Only runs of whitespace are collapsed.
- **`amount_paise`:** always greater than 0. `direction` is `debit` (money out of, or charged to, the Account) or `credit`.
- **`time`:** `HH:MM:SS` local time with no offset, or `null` (the default) when the source prints no time. Bank and card Statements leave it `null` unless they print a time column; payment-app Statements fill it.
- **Nullable fields:** `time`, `balance_paise`, `foreign_*`, `reference` and `source_id` may be `null`.
  - `reference`: a UPI RRN/UTR, or a NEFT/IMPS reference found in the description.
  - `source_id`: an id the issuer printed for this Transaction, if any.
- **Order:** Transactions are listed in the Statement's printed order.

### Check

```json
{ "name": "balance_chain", "outcome": "passed", "detail": { "rows": 40, "breaks": 0, "polarity": "normal" } }
```

| name | meaning |
|---|---|
| `balance_chain` | Each row's balance follows from the previous one. `polarity` is `normal` (savings) or `inverted` (loan or card outstanding) |
| `summary_totals` | The sums of debits and credits equal the printed totals, and opening ± sums equals closing |
| `row_counts` | The debit and credit counts equal the printed counts |

`outcome` is `passed`, `failed` or `skipped` (not enough printed figures). `detail` holds only numbers and enum strings. `summary_totals` and `row_counts` carry `"scope": "bill"` when the printed figures cover every card of a bill that lists several (each card is its own Statement): they are then Checked over the rows of all those Statements and reported on each.

### Payment history

A `payment_history` file (a payment app's export) fills `statements` with **one section per distinct instrument**, in first-seen order. Each section is an ordinary Statement:

- **`account`:** `{ "issuer": …, "kind": null, "mask": … }`.
  - `mask` is the instrument exactly as printed, without the app's leading words (e.g. `XXXXXX4321` or `ICICI R`), and is `null` when the row prints none. This is what the server remembers as the **instrument**: `instrument` is now the section's `account.mask`.
  - `issuer` is the issuer the vocabulary recognises in the instrument, else `null`.
  - `kind` is always `null`: an instrument does not say what kind of Account it is.
- **`period`:** the earliest and latest `date` among the section's rows.
- **`summary`:** every figure `null`, and `checks` is `[]`: the app prints no balances or totals.
- **`transactions`**, in printed order:
  - `date` and `time` (`HH:MM:SS`, `null` if the export prints no time);
  - `description`: the app's text, e.g. `Paid to SOME MERCHANT`;
  - `amount_paise` and `direction`;
  - `balance_paise`: always `null`;
  - `reference`: the UTR, else `null`;
  - `source_id`: the app's transaction id, else `null`.

```json
{
  "date": "2026-10-18",
  "time": "11:20:00",
  "description": "Paid to SOME MERCHANT",
  "amount_paise": 25000,
  "direction": "debit",
  "balance_paise": null,
  "foreign_amount": null,
  "foreign_currency": null,
  "reference": "512345678901",
  "source_id": "T2610181120000000000001"
}
```

## Changes

- **v2:** `version` is 2 in request and result. `payment_records` and `PaymentRecord` are gone; `payment_history` fills `statements` instead. Transactions gain `time`.
