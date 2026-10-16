//! The shared analysis query: `from`, `to`, `tz`.

mod common;

use common::{FakeToggl, call, time_app, time_db};

#[tokio::test]
async fn resolves_local_days_in_the_requested_zone() {
    let db = time_db("period_ok").await;
    let app = time_app(&db, &FakeToggl::new());

    let reply = call(
        &app,
        "GET",
        "/api/time/analysis/period?from=2026-01-10&to=2026-01-11&tz=Asia/Kolkata",
        None,
    )
    .await;

    assert_eq!(reply.status, 200);
    assert_eq!(reply.json["tz"], "Asia/Kolkata");
    assert_eq!(reply.json["days"].as_array().unwrap().len(), 2);
    assert_eq!(reply.json["days"][0]["date"], "2026-01-10");
    assert_eq!(reply.json["days"][0]["start"], "2026-01-09T18:30:00Z");
    assert_eq!(reply.json["days"][0]["end"], reply.json["days"][1]["start"]);
    db.drop().await;
}

#[tokio::test]
async fn bad_parameters_are_400_problems_with_a_code() {
    let db = time_db("period_bad").await;
    let app = time_app(&db, &FakeToggl::new());
    let cases = [
        (
            "from=2026-01-10&to=2026-01-11&tz=Nowhere/Land",
            "unknown_time_zone",
        ),
        ("from=2026-01-10&to=2026-01-11", "unknown_time_zone"),
        ("from=yesterday&to=2026-01-11&tz=UTC", "invalid_date"),
        ("from=2026-01-12&to=2026-01-11&tz=UTC", "period_reversed"),
        ("from=2024-01-01&to=2026-01-11&tz=UTC", "period_too_long"),
    ];
    for (query, code) in cases {
        let reply = call(
            &app,
            "GET",
            &format!("/api/time/analysis/period?{query}"),
            None,
        )
        .await;
        assert_eq!(reply.status, 400, "{query}");
        assert_eq!(reply.content_type, "application/problem+json");
        assert_eq!(reply.json["code"], code, "{query}");
    }
    db.drop().await;
}
