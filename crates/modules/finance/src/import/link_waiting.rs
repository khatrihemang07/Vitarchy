//! Linking Payment records to Transactions (ADR 0014): run after every confirm, so records
//! wait for the Statement that holds their Transaction and link as soon as it arrives.

use sqlx::PgConnection;

use crate::{
    link,
    linking::from_rows,
    repo::{payment_records, transactions},
};

/// Links every waiting Payment record that has a free Transaction; returns how many linked.
///
/// # Errors
/// The database failed.
pub async fn link_waiting(db: &mut PgConnection) -> Result<usize, sqlx::Error> {
    let records = payment_records::unlinked(&mut *db).await?;
    if records.is_empty() {
        return Ok(0);
    }
    let free = transactions::without_payment_record(&mut *db).await?;
    let pairs = link(
        &from_rows::stored_records(&records),
        &from_rows::transactions(&free),
    );
    let mut linked = 0;
    for (record_id, transaction_id) in pairs {
        if payment_records::link(&mut *db, record_id, transaction_id).await? {
            linked += 1;
        }
    }
    Ok(linked)
}
