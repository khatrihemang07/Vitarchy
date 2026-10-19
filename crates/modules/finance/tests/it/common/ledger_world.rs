//! Invented Accounts and Transactions for the Transactions list and Overview tests. Every
//! name and amount here is made up.

use super::{
    app::Env,
    results::{app_row, credit, history, instrument},
    transfers::{card_file, import, link, savings_file},
    tx,
};

/// Invented Accounts and rows (amounts in rupees):
/// - savings 1111: Rent 9,000 (day 3), Big out 10,000 (5), Coffee 250 (10), Salary credit
///   50,000 (12), Partial out 2,000 (15), and two debits of 700 (20, 21);
/// - card 2222: receipts of 4,000 (day 6) and 6,040 (8) for Big out, an Extra of 40;
/// - card 3333: a receipt of 500 (16) for Partial out, so 1,500 is still to come;
/// - card 2003: a UPI payment of 700 (25), printed again by a payment app as "SOME MERCHANT".
pub async fn world(label: &str) -> Env {
    let mut paid = tx(25, "PAID VIA UPI", 70_000);
    paid.reference = Some("512345678901".to_owned());
    let env = Env::new(
        label,
        vec![
            savings_file(
                "XXXX1111",
                vec![
                    tx(3, "Rent payment", 900_000),
                    tx(5, "Big out", 1_000_000),
                    tx(10, "Coffee shop", 25_000),
                    credit(12, "Salary July", 5_000_000),
                    tx(15, "Partial out", 200_000),
                    tx(20, "Pays 100% off_set", 70_000),
                    tx(21, "Pays twin", 70_000),
                ],
            ),
            card_file(
                "XXXX2222",
                vec![
                    credit(6, "Receipt one", 400_000),
                    credit(8, "Receipt two", 604_000),
                ],
            ),
            card_file("XXXX3333", vec![credit(16, "Receipt three", 50_000)]),
            card_file("XXXX2003", vec![paid]),
            history(vec![instrument(
                None,
                "XXXXXX2003",
                vec![app_row(
                    tx(25, "Paid to SOME MERCHANT", 70_000),
                    (11, 20),
                    Some("P1"),
                    Some("512345678901"),
                )],
            )]),
        ],
    )
    .await;
    for name in ["bank", "card two", "card three", "card app", "phone"] {
        import(&env, name).await;
    }
    let out = link(&env, "Big out", &["Receipt one", "Receipt two"]).await;
    assert_eq!(out.status, 201, "{}", out.text);
    let partial = link(&env, "Partial out", &["Receipt three"]).await;
    assert_eq!(partial.status, 201, "{}", partial.text);
    env
}

/// The id of the Account whose name contains `name_part`.
pub async fn account_id(env: &Env, name_part: &str) -> i64 {
    let accounts = env.get("/accounts").await;
    accounts
        .json
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["name"].as_str().unwrap().contains(name_part))
        .unwrap_or_else(|| panic!("no Account {name_part}"))["id"]
        .as_i64()
        .unwrap()
}
