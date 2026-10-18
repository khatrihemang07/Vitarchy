# statement-reader contract, version 1

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
  "version": 1,
  "file_path": "/abs/path/to/upload",
  "passwords": ["…", "…"],
  "decrypted_out": "/abs/path/where/a/decrypted/copy/goes"
}
```

`passwords` are the saved Statement passwords, in order. The helper tries each one, together with its case variants (as-is, upper, lower, capitalised), opening a fresh handle for every attempt. When the file was encrypted and unlocks, the helper writes the decrypted bytes to `decrypted_out`. When it was not encrypted, the helper writes nothing there.

## Result (stdout)

```json
{
  "version": 1,
  "outcome": "read",
  "format": "pdf",
  "encrypted": true,
  "unlocked_with": 2,
  "kind": "statement",
  "statements": [Statement],
  "payment_records": [PaymentRecord],
  "diagnostics": { "stage": null, "code": null, "pages": 4, "rows": 37 }
}
```

| field | values |
|---|---|
| `outcome` | `read` · `locked` (no password opened it) · `unsupported` (not a PDF/CSV/XLS/XLSX) · `no_table` (opened, but no transaction table found) |
| `format` | `pdf` · `csv` · `xls` · `xlsx` · `null` when unsupported |
| `unlocked_with` | index into `passwords` of the password whose variant worked, or `null` |
| `kind` | `statement` (bank or card) · `payment_history` (Payment history from a payment app such as PhonePe) · `null` unless `outcome` is `read` |
| `statements` | `[]` unless `kind` is `statement`. One file may hold several (consolidated savings + loans) |
| `payment_records` | `[]` unless `kind` is `payment_history` |
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
  "description": "UPI/DR/512345678901/DEMO GRO/HDFC/xyautopay./Indi",
  "amount_paise": 550000,
  "direction": "debit",
  "balance_paise": 8021778,
  "foreign_amount": "25.00",
  "foreign_currency": "USD",
  "reference": "512345678901",
  "addon_card": "5123XXXXXXXX1357",
  "source_id": null
}
```

- **`description`:** the text exactly as printed. Wrapped lines are joined: with a single space normally, and with nothing when the wrap split a token. Only runs of whitespace are collapsed.
- **`amount_paise`:** always greater than 0. `direction` is `debit` (money out of, or charged to, the Account) or `credit`.
- **Nullable fields:** `balance_paise`, `foreign_*`, `reference`, `addon_card` and `source_id` may be `null`.
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

`outcome` is `passed`, `failed` or `skipped` (not enough printed figures). `detail` holds only numbers and enum strings.

### PaymentRecord

```json
{
  "occurred_at": "2026-10-18T11:20:00",
  "description": "Paid to SOME MERCHANT",
  "amount_paise": 25000,
  "direction": "debit",
  "source_id": "T2610181120…",
  "utr": "512345678901",
  "instrument": "XXXXXX4321",
  "instrument_issuer": null,
  "instrument_digits": "4321"
}
```

`instrument` is the funding account's mask or label exactly as printed (e.g. `ICICI R`). `instrument_issuer` is the issuer the vocabulary recognises in it (e.g. `ICICI Bank`), else `null`. `instrument_digits` holds its trailing visible digits, else `null`. The server links a Payment record only to Transactions of Accounts that agree with these. When both are `null`, it links by UTR alone.

- `occurred_at` is local time with no offset.
