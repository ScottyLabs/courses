//! Skeleton behavior: no successful placeholder data and no personal writes.
use super::*;

#[tokio::test]
async fn skeleton_returns_explicit_json_error() {
    let response = not_implemented().into_response();
    assert_eq!(response.status(), StatusCode::NOT_IMPLEMENTED);
    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let body: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(body["message"], "Backend endpoint not implemented");
}

#[tokio::test]
async fn schedule_stub_preserves_existing_database_records() {
    let root = std::env::temp_dir().join(format!("courses-skeleton-{}", uuid::Uuid::new_v4()));
    let users = root.join("users.sqlite");
    let backend = Backend::open(
        root.join("catalog"),
        users.clone(),
        "http://localhost".into(),
    )
    .unwrap();
    backend
        .users
        .lock()
        .unwrap()
        .execute(
            "INSERT INTO schedules(id,owner,payload) VALUES('existing','owner','{}')",
            [],
        )
        .unwrap();
    let response = create_schedule(
        Err(StatusCode::UNAUTHORIZED),
        Extension(backend.clone()),
        Json(json!({"name":"new"})),
    )
    .await
    .unwrap_err()
    .into_response();
    assert_eq!(response.status(), StatusCode::NOT_IMPLEMENTED);
    drop(backend);
    let reopened = Backend::open(root.join("catalog"), users, "http://localhost".into()).unwrap();
    let db = reopened.users.lock().unwrap();
    let count: i64 = db
        .query_row(
            "SELECT count(*) FROM schedules WHERE id='existing' AND owner='owner'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);
    let total: i64 = db
        .query_row("SELECT count(*) FROM schedules", [], |row| row.get(0))
        .unwrap();
    assert_eq!(total, 1);
    drop(db);
    drop(reopened);
    std::fs::remove_dir_all(root).unwrap();
}
