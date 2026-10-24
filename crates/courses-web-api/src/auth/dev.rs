//! Development sign-in (ADR 26), compiled only with the `dev-auth` feature. When `APP_URL` is `http://localhost...`
//! and `DEV_USER` names an Andrew ID, every request is that verified CMU user, with no Keycloak involved.
//!
//! `/auth/login?as=<andrew>` switches to another user and `/auth/logout` signs out, through the `cw_dev_user`
//! cookie, so the website's signed-out states and friends between two users can be tried in one browser.

use anyhow::bail;
use axum::{
    http::{HeaderValue, header, request::Parts},
    response::{IntoResponse, Redirect, Response},
};

use super::{CurrentUser, member};
use crate::error::{ApiError, ApiResult};

const COOKIE: &str = "cw_dev_user";
const SIGNED_OUT: &str = "signed-out";

pub struct DevAuth {
    andrew_id: String,
    name: Option<String>,
}

/// `http://localhost`, optionally with a port or path. `http://localhost.example.com` is not.
pub fn is_localhost_http(app_url: &str) -> bool {
    app_url
        .strip_prefix("http://localhost")
        .is_some_and(|rest| rest.is_empty() || rest.starts_with(':') || rest.starts_with('/'))
}

fn valid_andrew_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-')
}

impl DevAuth {
    /// The development user, when `DEV_USER` is set. Refuses to start when `DEV_USER` is set for a non-local site.
    pub fn from_env(app_url: &str) -> anyhow::Result<Option<Self>> {
        let Some(user) = std::env::var("DEV_USER")
            .ok()
            .filter(|u| !u.trim().is_empty())
        else {
            return Ok(None);
        };
        if !is_localhost_http(app_url) {
            bail!(
                "DEV_USER is set, but APP_URL ({app_url}) isn't http://localhost; development sign-in only runs locally"
            );
        }
        let andrew_id = user
            .trim()
            .split('@')
            .next()
            .unwrap_or_default()
            .to_ascii_lowercase();
        if !valid_andrew_id(&andrew_id) {
            bail!("DEV_USER must be an Andrew ID, such as `jdoe`");
        }
        Ok(Some(Self {
            andrew_id,
            name: std::env::var("DEV_USER_NAME")
                .ok()
                .filter(|n| !n.trim().is_empty()),
        }))
    }

    pub fn andrew_id(&self) -> &str {
        &self.andrew_id
    }

    pub fn user(&self, parts: &Parts) -> ApiResult<CurrentUser> {
        let chosen = parts
            .headers
            .get_all(header::COOKIE)
            .iter()
            .filter_map(|v| v.to_str().ok())
            .flat_map(|v| v.split(';'))
            .find_map(|c| c.trim().strip_prefix(COOKIE)?.strip_prefix('='))
            .map(str::to_owned);
        let (andrew_id, name) = match chosen.as_deref() {
            Some(SIGNED_OUT) => return Err(ApiError::auth()),
            Some(other) if valid_andrew_id(other) && other != self.andrew_id => {
                (other.to_owned(), None)
            }
            _ => (self.andrew_id.clone(), self.name.clone()),
        };
        member(
            format!("dev:{andrew_id}"),
            Some(&format!("{andrew_id}@andrew.cmu.edu")),
            true,
            name,
            vec![],
        )
    }
}

fn cookie(value: &str, max_age: Option<u32>) -> HeaderValue {
    let age = max_age
        .map(|a| format!("; Max-Age={a}"))
        .unwrap_or_default();
    HeaderValue::from_str(&format!(
        "{COOKIE}={value}; Path=/; HttpOnly; SameSite=Lax{age}"
    ))
    .expect("valid cookie")
}

pub fn signed_out_cookie() -> HeaderValue {
    cookie(SIGNED_OUT, None)
}

/// Signs in as `as_user`, or as `DEV_USER` when none is given.
pub fn login(as_user: Option<&str>, target: &str) -> Response {
    let value = match as_user.map(str::to_ascii_lowercase) {
        Some(id) if valid_andrew_id(&id) => cookie(&id, None),
        _ => cookie("", Some(0)),
    };
    let mut response = Redirect::to(target).into_response();
    response.headers_mut().append(header::SET_COOKIE, value);
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_plain_localhost_counts() {
        assert!(is_localhost_http("http://localhost"));
        assert!(is_localhost_http("http://localhost:5173"));
        assert!(is_localhost_http("http://localhost/app"));
        assert!(!is_localhost_http("https://localhost:5173"));
        assert!(!is_localhost_http("http://localhost.evil.test"));
        assert!(!is_localhost_http("http://localhostevil.test"));
        assert!(!is_localhost_http("https://cmu.courses"));
    }

    #[test]
    fn cookie_switches_and_signs_out() {
        let dev = DevAuth {
            andrew_id: "alice".into(),
            name: Some("Alice A".into()),
        };
        let parts = |cookie: Option<&str>| {
            let mut req = axum::http::Request::builder();
            if let Some(c) = cookie {
                req = req.header(header::COOKIE, c);
            }
            req.body(()).unwrap().into_parts().0
        };
        let me = dev.user(&parts(None)).unwrap();
        assert_eq!(
            (me.andrew_id.as_str(), me.name.as_str()),
            ("alice", "Alice A")
        );
        assert_eq!(me.email, "alice@andrew.cmu.edu");
        let bob = dev.user(&parts(Some("x=1; cw_dev_user=bob"))).unwrap();
        assert_eq!(bob.subject, "dev:bob");
        assert!(dev.user(&parts(Some("cw_dev_user=signed-out"))).is_err());
        assert_eq!(
            dev.user(&parts(Some("cw_dev_user=BAD!")))
                .unwrap()
                .andrew_id,
            "alice"
        );
    }
}
