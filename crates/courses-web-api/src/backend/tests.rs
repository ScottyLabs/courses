use super::*;
fn fixture() -> Backend {
    let root = std::env::temp_dir().join(format!("courses-test-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&root).unwrap();
    for audience in ["public", "member"] {
        let db = Connection::open(root.join(format!("{audience}.sqlite"))).unwrap();
        db.execute_batch("CREATE TABLE catalog_terms(id TEXT,sort_order INTEGER,payload_json TEXT); INSERT INTO catalog_terms VALUES('F26',0,'{}'); CREATE TABLE catalog_courses(term_id TEXT,code TEXT,payload_json TEXT); CREATE TABLE catalog_instructors(id TEXT,name TEXT,department TEXT,payload_json TEXT);").unwrap();
        db.execute(
            "INSERT INTO catalog_courses VALUES('F26','15-122',?)",
            [json!({"code":"15-122","sections":[{"id":"s1"}]}).to_string()],
        )
        .unwrap();
    }
    Backend::open(
        root.clone(),
        root.join("users.sqlite"),
        "https://courses.test".into(),
    )
    .unwrap()
}
fn user(id: &str) -> Result<CurrentUser, StatusCode> {
    Ok(CurrentUser {
        subject: id.into(),
        name: id.into(),
        groups: vec![],
        andrew_id: id.into(),
    })
}
fn body() -> Value {
    json!({"name":"Fall plan","term_id":"F26"})
}
#[tokio::test]
async fn personal_records_survive_restart_and_enforce_ownership() {
    let b = fixture();
    let created = create_schedule(user("alice"), Extension(b.clone()), Json(body()))
        .await
        .unwrap()
        .0;
    let id = created["id"].as_str().unwrap().to_owned();
    assert_eq!(
        delete_schedule(user("bob"), Extension(b.clone()), Path(id.clone()))
            .await
            .unwrap_err()
            .0,
        StatusCode::NOT_FOUND
    );
    let reopened = Backend::open(
        b.catalog.clone(),
        b.catalog.join("users.sqlite"),
        b.origin.clone(),
    )
    .unwrap();
    assert_eq!(
        schedules(user("alice"), Extension(reopened))
            .await
            .unwrap()
            .0
            .as_array()
            .unwrap()
            .len(),
        1
    );
    std::fs::remove_dir_all(&b.catalog).unwrap();
}
#[tokio::test]
async fn saved_courses_survive_restart_and_are_owner_scoped() {
    let b = fixture();
    let catalog = Connection::open(b.catalog.join("member.sqlite")).unwrap();
    catalog
        .execute(
            "INSERT INTO catalog_courses VALUES('', '15-122', ?)",
            [json!({"code":"15-122","sections":[]}).to_string()],
        )
        .unwrap();
    let _ = save_course(
        user("alice"),
        Extension(b.clone()),
        Path("15-122".into()),
        Json(json!({"status":"saved"})),
    )
    .await
    .unwrap();
    let _ = save_course(
        user("alice"),
        Extension(b.clone()),
        Path("15-122".into()),
        Json(json!({"status":"taken","term_id":"F26"})),
    )
    .await
    .unwrap();
    let reopened = Backend::open(
        b.catalog.clone(),
        b.catalog.join("users.sqlite"),
        b.origin.clone(),
    )
    .unwrap();
    assert_eq!(
        saved(
            user("alice"),
            Extension(reopened.clone()),
            Query(HashMap::new())
        )
        .await
        .unwrap()
        .0
        .as_array()
        .unwrap()
        .len(),
        1
    );
    assert!(
        saved(
            user("bob"),
            Extension(reopened.clone()),
            Query(HashMap::new())
        )
        .await
        .unwrap()
        .0
        .as_array()
        .unwrap()
        .is_empty()
    );
    unsave_course(
        user("alice"),
        Extension(reopened.clone()),
        Path("15-122".into()),
        Query(HashMap::new()),
    )
    .await
    .unwrap();
    assert!(
        saved(
            user("alice"),
            Extension(reopened.clone()),
            Query(HashMap::new())
        )
        .await
        .unwrap()
        .0
        .as_array()
        .unwrap()
        .is_empty()
    );
    assert_eq!(
        saved(
            user("alice"),
            Extension(reopened),
            Query(HashMap::from([("status".into(), "taken".into())]))
        )
        .await
        .unwrap()
        .0
        .as_array()
        .unwrap()
        .len(),
        1
    );
    std::fs::remove_dir_all(&b.catalog).unwrap();
}
#[tokio::test]
async fn stale_revisions_and_wrong_sections_are_rejected() {
    let b = fixture();
    let mut v = create_schedule(user("alice"), Extension(b.clone()), Json(body()))
        .await
        .unwrap()
        .0;
    let id = v["id"].as_str().unwrap().to_owned();
    v["items"] = json!([{"course_code":"15-122","section_ids":["s1"]}]);
    let _ = save_schedule(
        user("alice"),
        Extension(b.clone()),
        Path(id.clone()),
        Json(v.clone()),
    )
    .await
    .unwrap();
    assert_eq!(
        save_schedule(
            user("alice"),
            Extension(b.clone()),
            Path(id.clone()),
            Json(v.clone())
        )
        .await
        .unwrap_err()
        .0,
        StatusCode::CONFLICT
    );
    v["items"][0]["section_ids"] = json!(["other-course"]);
    assert_eq!(
        save_schedule(user("alice"), Extension(b.clone()), Path(id), Json(v))
            .await
            .unwrap_err()
            .0,
        StatusCode::BAD_REQUEST
    );
    std::fs::remove_dir_all(&b.catalog).unwrap();
}
#[tokio::test]
async fn shares_can_be_revoked_and_exclude_owner() {
    let b = fixture();
    let v = create_schedule(user("alice"), Extension(b.clone()), Json(body()))
        .await
        .unwrap()
        .0;
    let id = v["id"].as_str().unwrap().to_owned();
    let grant = share(user("alice"), Extension(b.clone()), Path(id.clone()))
        .await
        .unwrap()
        .0;
    let token = grant["share_id"].as_str().unwrap().to_owned();
    let view = shared(Extension(b.clone()), Path(token.clone()))
        .await
        .unwrap()
        .0;
    assert!(view.get("owner").is_none());
    assert_eq!(view["id"], "shared");
    revoke_share(user("alice"), Extension(b.clone()), Path(id))
        .await
        .unwrap();
    assert_eq!(
        shared(Extension(b.clone()), Path(token))
            .await
            .unwrap_err()
            .0,
        StatusCode::NOT_FOUND
    );
    std::fs::remove_dir_all(&b.catalog).unwrap();
}
#[tokio::test]
async fn anonymous_cannot_read_instructors_or_personal_data() {
    let b = fixture();
    assert_eq!(
        instructors(
            Err(StatusCode::UNAUTHORIZED),
            Extension(b.clone()),
            Query(HashMap::new())
        )
        .await
        .unwrap_err()
        .0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        schedules(Err(StatusCode::FORBIDDEN), Extension(b.clone()))
            .await
            .unwrap_err()
            .0,
        StatusCode::FORBIDDEN
    );
    assert!(course_filter(&HashMap::from([("min_fce".into(), "4".into())]), false).is_err());
    std::fs::remove_dir_all(&b.catalog).unwrap();
}
#[tokio::test]
async fn origin_is_required_for_cookie_mutations() {
    use tower::ServiceExt;
    let b = fixture();
    let app = axum::Router::new()
        .route("/write", axum::routing::post(|| async { "ok" }))
        .layer(axum::middleware::from_fn(guard))
        .layer(Extension(b.clone()));
    for origin in [
        None,
        Some("https://evil.test"),
        Some("https://courses.test"),
    ] {
        let mut req = Request::builder().method("POST").uri("/write");
        if let Some(origin) = origin {
            req = req.header(header::ORIGIN, origin)
        }
        let result = app
            .clone()
            .oneshot(req.body(axum::body::Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(
            result.status(),
            if origin == Some("https://courses.test") {
                StatusCode::OK
            } else {
                StatusCode::FORBIDDEN
            }
        );
    }
    std::fs::remove_dir_all(&b.catalog).unwrap();
}

#[tokio::test]
async fn friendship_requires_consent_and_invites_are_single_use() {
    let b = fixture();
    register_user(&b, &user("alice").unwrap()).unwrap();
    register_user(&b, &user("bob").unwrap()).unwrap();
    let request = request_friend(
        user("alice"),
        Extension(b.clone()),
        Json(json!({"andrew_id":"bob"})),
    )
    .await
    .unwrap()
    .0;
    let id = request["id"].as_str().unwrap().to_owned();
    assert_eq!(
        accept_friend(user("alice"), Extension(b.clone()), Path(id.clone()))
            .await
            .unwrap_err()
            .0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        friends(user("bob"), Extension(b.clone())).await.unwrap().0[0]["status"],
        "incoming"
    );
    accept_friend(user("bob"), Extension(b.clone()), Path(id.clone()))
        .await
        .unwrap();
    assert_eq!(
        friends(user("alice"), Extension(b.clone()))
            .await
            .unwrap()
            .0[0]["status"],
        "accepted"
    );
    assert_eq!(
        friends(user("alice"), Extension(b.clone()))
            .await
            .unwrap()
            .0[0]["shared_courses"],
        json!([])
    );
    remove_friend(user("bob"), Extension(b.clone()), Path(id))
        .await
        .unwrap();
    let link = invite(user("alice"), Extension(b.clone())).await.unwrap().0;
    let token = link["id"].as_str().unwrap().to_owned();
    redeem_invite(user("bob"), Extension(b.clone()), Path(token.clone()))
        .await
        .unwrap();
    assert_eq!(
        redeem_invite(user("bob"), Extension(b.clone()), Path(token))
            .await
            .unwrap_err()
            .0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        friends(user("bob"), Extension(b.clone())).await.unwrap().0[0]["status"],
        "accepted"
    );
    std::fs::remove_dir_all(&b.catalog).unwrap();
}

#[tokio::test]
async fn finals_are_private_and_deleting_another_users_import_is_impossible() {
    let b = fixture();
    user_db(&b)
        .unwrap()
        .execute(
            "INSERT INTO final_imports VALUES('alice','F26','[{\"course_code\":\"15-122\"}]')",
            [],
        )
        .unwrap();
    let query = || Query(HashMap::from([("term".into(), "F26".into())]));
    assert_eq!(
        finals(Err(StatusCode::UNAUTHORIZED), Extension(b.clone()), query())
            .await
            .unwrap_err()
            .0,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        finals(user("bob"), Extension(b.clone()), query())
            .await
            .unwrap()
            .0,
        json!([])
    );
    clear_finals(user("bob"), Extension(b.clone()), query())
        .await
        .unwrap();
    let reopened = Backend::open(
        b.catalog.clone(),
        b.catalog.join("users.sqlite"),
        b.origin.clone(),
    )
    .unwrap();
    assert_eq!(
        finals(user("alice"), Extension(reopened), query())
            .await
            .unwrap()
            .0
            .as_array()
            .unwrap()
            .len(),
        1
    );
    std::fs::remove_dir_all(&b.catalog).unwrap();
}
