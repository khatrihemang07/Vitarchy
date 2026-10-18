//! The coded problem responses of Finance: core's problem fields plus a machine `code` the
//! web app acts on, and the table of refusals the handlers can answer with.

mod body;

pub use body::{ProblemBody, problem_response};

/// A request Finance refuses, as data: the status, the code and what to tell the person.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Refusal {
    /// HTTP status.
    pub status: u16,
    /// Stable code.
    pub code: &'static str,
    /// Fixed explanation (never quotes what the person entered).
    pub detail: &'static str,
}

const fn refusal(status: u16, code: &'static str, detail: &'static str) -> Refusal {
    Refusal {
        status,
        code,
        detail,
    }
}

impl Refusal {
    /// No such Statement.
    pub const STATEMENT_NOT_FOUND: Self =
        refusal(404, "statement_not_found", "There is no such Statement.");
    /// No such Transaction.
    pub const TRANSACTION_NOT_FOUND: Self = refusal(
        404,
        "transaction_not_found",
        "There is no such Transaction.",
    );
    /// The file was not read, so there is nothing to confirm or compare.
    pub const NOT_READABLE: Self = refusal(
        409,
        "statement_not_readable",
        "This file was not read, so it cannot be confirmed. Add a password or Read again first.",
    );
    /// The Statement is already confirmed.
    pub const CONFIRMED: Self = refusal(
        409,
        "statement_confirmed",
        "This Statement is already confirmed.",
    );
    /// An unconfirmed Statement already has Printings on Transactions.
    pub const HAS_ROWS: Self = refusal(
        409,
        "statement_has_rows",
        "Transactions already hold rows from this Statement, so it cannot be discarded.",
    );
    /// The Statement is not confirmed yet.
    pub const UNCONFIRMED: Self = refusal(
        409,
        "statement_unconfirmed",
        "This Statement is not confirmed yet.",
    );
    /// The engine now finds different Accounts in the file.
    pub const SECTIONS_CHANGED: Self = refusal(
        409,
        "statement_sections_changed",
        "The file now lists different Accounts than when it was confirmed.",
    );
    /// A confirmed Statement lost its sections and one cannot be placed on a single Account.
    pub const SECTIONS_UNPLACED: Self = refusal(
        409,
        "statement_sections_unplaced",
        "This file lists an Account that cannot be matched to your Transactions, so nothing was changed.",
    );
    /// A chosen Account does not exist.
    pub const ACCOUNT_NOT_FOUND: Self =
        refusal(404, "account_not_found", "There is no such Account.");
    /// The confirm body does not have one choice per Account in the Statement.
    pub const SECTIONS_MISMATCH: Self = refusal(
        400,
        "sections_mismatch",
        "Choose an Account for every Account listed in the Statement.",
    );
    /// A choice names neither or both of an Account and a new Account.
    pub const INVALID_SECTION: Self = refusal(
        400,
        "invalid_section",
        "Each choice needs either account_id or new_account.",
    );
    /// The proposal lacks an issuer or kind and no new Account was given.
    pub const NEW_ACCOUNT_REQUIRED: Self = refusal(
        400,
        "new_account_required",
        "This Account was not recognised: give its issuer, kind and name.",
    );
    /// A new Account has an empty issuer or name, or an unknown kind.
    pub const INVALID_NEW_ACCOUNT: Self = refusal(
        400,
        "invalid_new_account",
        "A new Account needs an issuer, a name and a kind of savings, credit_card or loan.",
    );
    /// The upload held no files.
    pub const NO_FILES: Self = refusal(400, "no_files", "Choose at least one file to upload.");
    /// The upload could not be read as a form.
    pub const BAD_UPLOAD: Self = refusal(400, "bad_upload", "The upload could not be read.");
    /// The upload is bigger than the limit.
    pub const UPLOAD_TOO_LARGE: Self = refusal(413, "upload_too_large", "The upload is too large.");
    /// Several Accounts fit a proposal and none was chosen.
    pub const AMBIGUOUS_ACCOUNT: Self = refusal(
        400,
        "account_ambiguous",
        "Several Accounts fit this one: choose which.",
    );
    /// A name or label is empty.
    pub const EMPTY_NAME: Self = refusal(400, "empty_name", "The name cannot be empty.");
    /// A password is empty.
    pub const EMPTY_SECRET: Self = refusal(400, "empty_secret", "The password cannot be empty.");
    /// A query parameter is invalid.
    pub const INVALID_QUERY: Self = refusal(
        400,
        "invalid_query",
        "The query is invalid: check the status, dates, limit and offset.",
    );
}
