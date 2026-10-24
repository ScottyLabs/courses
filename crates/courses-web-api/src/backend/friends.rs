//! Mutual friendship consent. Relationships never implicitly disclose schedules.
use super::*;
use chrono::Utc;

pub fn register_user(b: &Backend, user: &CurrentUser) -> Result<(), ApiError> {
    user_db(b)?.execute("INSERT INTO profiles(subject,andrew_id,name) VALUES(?,?,?) ON CONFLICT(subject) DO UPDATE SET andrew_id=excluded.andrew_id,name=excluded.name", params![user.subject,user.andrew_id,user.name])?;
    Ok(())
}

pub async fn friends(
    user: Result<CurrentUser, StatusCode>,
    Extension(b): Extension<Backend>,
) -> ApiResult {
    let user = require(user)?;
    register_user(&b, &user)?;
    let db = user_db(&b)?;
    let mut stmt = db.prepare("SELECT f.id,f.requester,f.recipient,f.accepted,p.name FROM friendships f JOIN profiles p ON p.subject=CASE WHEN f.requester=? THEN f.recipient ELSE f.requester END WHERE f.requester=? OR f.recipient=?")?;
    let mut list = stmt.query_map(params![user.subject,user.subject,user.subject], |r| {
        let from: String = r.get(1)?;
        let accepted: bool = r.get(3)?;
        Ok(json!({"id":r.get::<_,String>(0)?,"name":r.get::<_,String>(4)?,"status":if accepted {"accepted"} else if from==user.subject {"outgoing"} else {"incoming"},"shared_courses":[]}))
    })?.collect::<Result<Vec<_>,_>>()?;
    for friend in &mut list {
        if friend["status"] != "accepted" {
            continue;
        }
        let id = friend["id"].as_str().unwrap();
        let shared = rows(
            &db,
            "SELECT json_quote(a.code) FROM friendships f JOIN profiles p ON p.subject=f.requester JOIN profiles q ON q.subject=f.recipient JOIN saved_courses a ON a.owner=p.subject JOIN saved_courses z ON z.owner=q.subject AND a.code=z.code AND a.term_id=z.term_id WHERE f.id=? AND p.share_courses=1 AND q.share_courses=1 AND a.status IN ('taking','taken') AND z.status IN ('taking','taken') GROUP BY a.code",
            vec![val(id)],
        )?;
        friend["shared_courses"] = json!(shared);
    }
    Ok(Json(json!(list)))
}

pub async fn request_friend(
    user: Result<CurrentUser, StatusCode>,
    Extension(b): Extension<Backend>,
    Json(v): Json<Value>,
) -> ApiResult {
    let user = require(user)?;
    register_user(&b, &user)?;
    let andrew = v["andrew_id"]
        .as_str()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    if andrew.is_empty()
        || andrew.len() > 64
        || !andrew
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err(bad("Invalid Andrew ID"));
    }
    let db = user_db(&b)?;
    let targets = rows(
        &db,
        "SELECT json_object('subject',subject,'name',name) FROM profiles WHERE andrew_id=?",
        vec![val(&andrew)],
    )?;
    let target = targets.first().ok_or_else(|| {
        bad("No account found; ask your friend to sign in first or use an invite link")
    })?;
    let recipient = target["subject"].as_str().unwrap();
    let id = request(&db, &user.subject, recipient)?;
    Ok(Json(
        json!({"id":id,"name":target["name"],"status":"outgoing","shared_courses":[]}),
    ))
}

fn request(db: &Connection, requester: &str, recipient: &str) -> Result<String, ApiError> {
    if requester == recipient {
        return Err(bad("You cannot add yourself"));
    }
    let pair = if requester < recipient {
        format!("{requester}\n{recipient}")
    } else {
        format!("{recipient}\n{requester}")
    };
    let id = uuid::Uuid::new_v4().to_string();
    let exists: bool = db.query_row(
        "SELECT EXISTS(SELECT 1 FROM friendships WHERE pair=?)",
        [&pair],
        |r| r.get(0),
    )?;
    if exists {
        return Err(bad("A friendship or request already exists"));
    }
    db.execute(
        "INSERT INTO friendships(id,pair,requester,recipient,accepted) VALUES(?,?,?,?,0)",
        params![id, pair, requester, recipient],
    )?;
    Ok(id)
}

pub async fn accept_friend(
    user: Result<CurrentUser, StatusCode>,
    Extension(b): Extension<Backend>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let user = require(user)?;
    if user_db(&b)?.execute(
        "UPDATE friendships SET accepted=1 WHERE id=? AND recipient=? AND accepted=0",
        params![id, user.subject],
    )? == 0
    {
        return Err(missing());
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn remove_friend(
    user: Result<CurrentUser, StatusCode>,
    Extension(b): Extension<Backend>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let user = require(user)?;
    if user_db(&b)?.execute(
        "DELETE FROM friendships WHERE id=? AND (requester=? OR recipient=?)",
        params![id, user.subject, user.subject],
    )? == 0
    {
        return Err(missing());
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn invite(
    user: Result<CurrentUser, StatusCode>,
    Extension(b): Extension<Backend>,
) -> ApiResult {
    let user = require(user)?;
    register_user(&b, &user)?;
    let token = uuid::Uuid::new_v4().to_string();
    let db = user_db(&b)?;
    db.execute(
        "DELETE FROM friend_invites WHERE owner=? OR expires_at<?",
        params![user.subject, Utc::now().timestamp()],
    )?;
    db.execute(
        "INSERT INTO friend_invites(token,owner,expires_at) VALUES(?,?,?)",
        params![
            token,
            user.subject,
            (Utc::now() + chrono::Duration::days(7)).timestamp()
        ],
    )?;
    Ok(Json(
        json!({"id":token,"url":format!("{}/?view=profile&invite={token}",b.origin)}),
    ))
}

pub async fn revoke_invite(
    user: Result<CurrentUser, StatusCode>,
    Extension(b): Extension<Backend>,
    Path(token): Path<String>,
) -> Result<StatusCode, ApiError> {
    let user = require(user)?;
    user_db(&b)?.execute(
        "DELETE FROM friend_invites WHERE token=? AND owner=?",
        params![token, user.subject],
    )?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn redeem_invite(
    user: Result<CurrentUser, StatusCode>,
    Extension(b): Extension<Backend>,
    Path(token): Path<String>,
) -> Result<StatusCode, ApiError> {
    let user = require(user)?;
    register_user(&b, &user)?;
    let mut db = user_db(&b)?;
    let tx = db.transaction()?;
    let invites = rows(
        &tx,
        "SELECT json_object('owner',owner) FROM friend_invites WHERE token=? AND expires_at>?",
        vec![val(&token), SqlValue::Integer(Utc::now().timestamp())],
    )?;
    let owner = invites
        .first()
        .and_then(|v| v["owner"].as_str())
        .ok_or_else(missing)?;
    let id = request(&tx, owner, &user.subject)?;
    tx.execute("UPDATE friendships SET accepted=1 WHERE id=?", [id])?;
    tx.execute("DELETE FROM friend_invites WHERE token=?", [token])?;
    tx.commit()?;
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn duplicate_and_self_requests_are_rejected() {
        let db = Connection::open_in_memory().unwrap();
        db.execute_batch("CREATE TABLE friendships(id TEXT,pair TEXT UNIQUE,requester TEXT,recipient TEXT,accepted INTEGER);").unwrap();
        assert!(request(&db, "alice", "alice").is_err());
        request(&db, "alice", "bob").unwrap();
        assert!(request(&db, "bob", "alice").is_err());
        assert!(
            !db.query_row("SELECT accepted FROM friendships", [], |r| r
                .get::<_, bool>(0))
                .unwrap()
        );
    }
}

/// A separate opt-in; accepting a friendship never toggles this preference.
pub async fn friend_visibility(
    user: Result<CurrentUser, StatusCode>,
    Extension(b): Extension<Backend>,
    Json(v): Json<Value>,
) -> ApiResult {
    let user = require(user)?;
    register_user(&b, &user)?;
    let shared = v["share_courses"]
        .as_bool()
        .ok_or_else(|| bad("share_courses must be a boolean"))?;
    user_db(&b)?.execute(
        "UPDATE profiles SET share_courses=? WHERE subject=?",
        params![shared, user.subject],
    )?;
    Ok(Json(json!({"share_courses":shared})))
}
