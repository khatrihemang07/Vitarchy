//! The write-only list of Statement passwords.

use crate::common::app::Env;
use crate::common::results::locked;

#[tokio::test]
async fn a_saved_password_is_listed_by_label_and_never_by_secret() {
    let env = Env::new("pw_list", vec![locked()]).await;

    let saved = env
        .post(
            "/statement-passwords",
            r#"{"label":"  Card  ","secret":"  s3cr3t-value "}"#,
        )
        .await;
    let listed = env.get("/statement-passwords").await;

    assert_eq!(saved.status, 201, "{}", saved.text);
    assert_eq!(saved.json["label"], "Card");
    assert!(saved.json["created_at"].is_string() && saved.json["last_used_at"].is_null());
    assert_eq!(listed.json.as_array().unwrap().len(), 1);
    for text in [&saved.text, &listed.text] {
        assert!(
            !text.contains("s3cr3t") && !text.contains("secret"),
            "{text}"
        );
    }
    let stored: String = sqlx::query_scalar("SELECT secret FROM finance.statement_passwords")
        .fetch_one(&env.db.pool)
        .await
        .unwrap();
    assert_eq!(stored, "s3cr3t-value", "the secret is trimmed");
    env.finish().await;
}

#[tokio::test]
async fn an_empty_label_or_secret_is_refused_with_a_code() {
    let env = Env::new("pw_empty", vec![locked()]).await;

    let no_label = env
        .post("/statement-passwords", r#"{"label":"  ","secret":"x"}"#)
        .await;
    let no_secret = env
        .post("/statement-passwords", r#"{"label":"a","secret":"   "}"#)
        .await;
    let malformed = env.post("/statement-passwords", r#"{"label":"a"}"#).await;

    assert_eq!(
        (no_label.status, no_label.json["code"].as_str()),
        (400, Some("empty_name"))
    );
    assert_eq!(
        (no_secret.status, no_secret.json["code"].as_str()),
        (400, Some("empty_secret"))
    );
    assert!(
        malformed.status == 400 || malformed.status == 422,
        "{}",
        malformed.status
    );
    assert!(
        env.get("/statement-passwords")
            .await
            .json
            .as_array()
            .unwrap()
            .is_empty()
    );
    env.finish().await;
}

#[tokio::test]
async fn a_password_can_be_forgotten() {
    let env = Env::new("pw_delete", vec![locked()]).await;
    let id = env
        .post("/statement-passwords", r#"{"label":"a","secret":"b"}"#)
        .await
        .json["id"]
        .as_i64()
        .unwrap();

    let gone = env
        .json_call("DELETE", &format!("/statement-passwords/{id}"), None)
        .await;
    let again = env
        .json_call("DELETE", &format!("/statement-passwords/{id}"), None)
        .await;

    assert_eq!(gone.status, 204);
    assert_eq!(again.status, 404);
    assert!(
        env.get("/statement-passwords")
            .await
            .json
            .as_array()
            .unwrap()
            .is_empty()
    );
    env.finish().await;
}
