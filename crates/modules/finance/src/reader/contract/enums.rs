//! The contract's closed sets of strings.

use serde::{Deserialize, Serialize};

use super::wire_enum::wire_enum;

wire_enum! {
    /// What became of the file.
    Outcome {
        /// Opened and a table was found.
        Read = "read",
        /// No saved Statement password opened it.
        Locked = "locked",
        /// Not a PDF, CSV, XLS or XLSX.
        Unsupported = "unsupported",
        /// Opened, but no transaction table was found.
        NoTable = "no_table",
    }
}

wire_enum! {
    /// The file's format.
    Format {
        /// PDF.
        Pdf = "pdf",
        /// CSV.
        Csv = "csv",
        /// Legacy spreadsheet.
        Xls = "xls",
        /// OOXML spreadsheet.
        Xlsx = "xlsx",
    }
}

wire_enum! {
    /// What a read file holds.
    Kind {
        /// A bank or card Statement.
        Statement = "statement",
        /// A payment app's history of Payments, as Payment records.
        PaymentHistory = "payment_history",
    }
}

wire_enum! {
    /// Which way money moved on the Account.
    Direction {
        /// Out of, or charged to, the Account.
        Debit = "debit",
        /// Into the Account.
        Credit = "credit",
    }
}

wire_enum! {
    /// What kind of Account.
    AccountKind {
        /// A savings account.
        Savings = "savings",
        /// A credit card.
        CreditCard = "credit_card",
        /// A loan.
        Loan = "loan",
    }
}

wire_enum! {
    /// How a Check came out.
    CheckOutcome {
        /// The figures agree.
        Passed = "passed",
        /// The figures disagree: a warning, not a rejection.
        Failed = "failed",
        /// Not enough printed figures to tell.
        Skipped = "skipped",
    }
}
