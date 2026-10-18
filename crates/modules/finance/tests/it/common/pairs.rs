//! Two overlapping Statements of one Account, confirmed in order.

use vitarchy_finance::reader::contract::Transaction;

use super::{
    app::Env,
    results::{card_section, read},
};

const MASK: &str = "XXXX2003";

/// A one-section file of this Account.
pub fn file(rows: Vec<Transaction>) -> vitarchy_finance::reader::ReaderResult {
    read(vec![card_section(MASK, rows)])
}

/// An Env whose reader answers `uploads` (one per uploaded file), then `later` (one per
/// Read again), in order; returns it with the id of each uploaded Statement (none confirmed).
pub async fn uploaded(
    label: &str,
    uploads: Vec<Vec<Transaction>>,
    later: Vec<Vec<Transaction>>,
) -> (Env, Vec<i64>) {
    let count = uploads.len();
    let env = Env::new(label, uploads.into_iter().chain(later).map(file).collect()).await;
    let mut ids = Vec::new();
    for n in 0..count {
        let bytes = format!("bytes{n}");
        ids.push(
            env.upload_one(&format!("file{n}.pdf"), bytes.as_bytes())
                .await,
        );
    }
    (env, ids)
}

/// [`uploaded`], with every upload confirmed in order.
pub async fn confirmed(
    label: &str,
    uploads: Vec<Vec<Transaction>>,
    later: Vec<Vec<Transaction>>,
) -> (Env, Vec<i64>) {
    let (env, ids) = uploaded(label, uploads, later).await;
    for id in &ids {
        let reply = env.confirm_proposed(*id, 1).await;
        assert_eq!(reply.status, 200, "{}", reply.text);
    }
    (env, ids)
}
