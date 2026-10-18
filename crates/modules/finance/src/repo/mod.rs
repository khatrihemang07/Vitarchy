//! Storage: sqlx queries, one file per table.

pub mod account_identifiers;
pub mod account_stats;
pub mod accounts;
pub mod confirm_lock;
pub mod counts;
pub mod passwords;
pub mod payment_records;
pub mod printings;
pub mod printings_list;
mod printings_load;
pub mod runs;
pub mod statement_accounts;
pub mod statement_changes;
pub mod statement_files;
pub mod statements;
pub mod transaction_page;
pub mod transactions;
pub mod transactions_read_again;
pub mod transactions_resolved;
