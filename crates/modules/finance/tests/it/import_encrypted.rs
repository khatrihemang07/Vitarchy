//! Locked files: the decrypted copy is the kept original, and a saved password opens a
//! locked Statement on Read again.

use crate::common::{
    app::Env,
    results::{card_section, locked, read, unlocked},
    tx,
};
use vitarchy_finance::{files::sha256_hex, reader::testing::FakeReader};

const SECRET: &str = "hunter2";

fn card() -> vitarchy_finance::reader::ReaderResult {
    read(vec![card_section("XXXX2003", vec![tx(2, "SHOP", 100)])])
}

async fn save_password(env: &Env) {
    let body = format!(r#"{{"label":"Main card","secret":"{SECRET}"}}"#);
    assert_eq!(env.post("/statement-passwords", &body).await.status, 201);
}

#[tokio::test]
async fn an_unlocked_upload_keeps_the_decrypted_copy_and_drops_the_encrypted_one() {
    let reader =
        FakeReader::with_results([unlocked(card())]).writing_decrypted(b"DECRYPTED".to_vec());
    let env = Env::with_reader("enc_store", reader).await;
    save_password(&env).await;

    let reply = env.upload(&[("locked.pdf", b"ENCRYPTED UPLOAD")]).await;

    let p = &reply.json[0];
    assert_eq!(p["outcome"], "read");
    assert_eq!(p["encrypted"], true);
    assert_eq!(p["unlocked_with_label"], "Main card");
    assert_eq!(
        std::fs::read(env.stored_file(&sha256_hex(b"DECRYPTED"))).unwrap(),
        b"DECRYPTED"
    );
    assert!(!env.stored_file(&sha256_hex(b"ENCRYPTED UPLOAD")).exists());
    assert!(
        !env.store()
            .scratch_path(&sha256_hex(b"ENCRYPTED UPLOAD"))
            .exists()
    );
    assert_eq!(env.reader.calls()[0].password_count, 1);
    env.finish().await;
}

#[tokio::test]
async fn re_uploading_the_encrypted_file_is_still_recognised() {
    let reader =
        FakeReader::with_results([unlocked(card())]).writing_decrypted(b"DECRYPTED".to_vec());
    let env = Env::with_reader("enc_dup", reader).await;
    save_password(&env).await;
    let id = env.upload_one("locked.pdf", b"ENCRYPTED UPLOAD").await;

    let again = env.upload(&[("locked.pdf", b"ENCRYPTED UPLOAD")]).await;

    assert_eq!(
        (
            again.json[0]["id"].as_i64(),
            again.json[0]["duplicate_file"].as_bool()
        ),
        (Some(id), Some(true))
    );
    assert_eq!(env.reader.calls().len(), 1);
    assert!(!env.stored_file(&sha256_hex(b"ENCRYPTED UPLOAD")).exists());
    env.finish().await;
}

#[tokio::test]
async fn the_password_that_opened_a_file_is_marked_used() {
    let reader =
        FakeReader::with_results([unlocked(card())]).writing_decrypted(b"DECRYPTED".to_vec());
    let env = Env::with_reader("enc_used", reader).await;
    save_password(&env).await;
    assert!(env.get("/statement-passwords").await.json[0]["last_used_at"].is_null());

    env.upload_one("locked.pdf", b"ENCRYPTED UPLOAD").await;

    assert!(env.get("/statement-passwords").await.json[0]["last_used_at"].is_string());
    env.finish().await;
}

#[tokio::test]
async fn adding_a_password_then_reading_again_opens_a_locked_statement() {
    let reader = FakeReader::with_results([locked(), unlocked(card())])
        .writing_decrypted(b"DECRYPTED".to_vec());
    let env = Env::with_reader("enc_retry", reader).await;
    let id = env.upload_one("locked.pdf", b"ENCRYPTED UPLOAD").await;
    assert_eq!(
        env.confirm_proposed(id, 0).await.json["code"],
        "statement_not_readable"
    );
    save_password(&env).await;

    let reply = env
        .post(&format!("/statements/{id}/read-again"), "{}")
        .await;

    assert_eq!(reply.status, 200, "{}", reply.text);
    let statement = &reply.json["statement"];
    assert_eq!(
        (statement["outcome"].as_str(), statement["status"].as_str()),
        (Some("read"), Some("unconfirmed"))
    );
    assert_eq!(statement["unlocked_with_label"], "Main card");
    assert_eq!(statement["sections"][0]["transaction_count"], 1);
    assert!(reply.json["sections"].as_array().unwrap().is_empty());
    assert_eq!(env.reader.calls()[1].password_count, 1);
    assert!(env.stored_file(&sha256_hex(b"DECRYPTED")).exists());
    assert!(!env.stored_file(&sha256_hex(b"ENCRYPTED UPLOAD")).exists());
    assert_eq!(env.confirm_proposed(id, 1).await.status, 200);
    env.finish().await;
}

#[tokio::test]
async fn reading_again_without_a_working_password_stays_locked() {
    let env = Env::new("enc_still_locked", vec![locked()]).await;
    let id = env.upload_one("locked.pdf", b"ENCRYPTED UPLOAD").await;

    let reply = env
        .post(&format!("/statements/{id}/read-again"), "{}")
        .await;

    assert_eq!(reply.status, 200);
    assert_eq!(reply.json["statement"]["outcome"], "locked");
    assert!(env.stored_file(&sha256_hex(b"ENCRYPTED UPLOAD")).exists());
    env.finish().await;
}
