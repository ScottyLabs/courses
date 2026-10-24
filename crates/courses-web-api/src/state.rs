//! Shared application state, passed to handlers as an `Extension`.

use std::{ops::Deref, sync::Arc};

use crate::{
    auth::AuthMode, catalog::Catalog, db::UserDb, ratelimit::Limiter, seo::Site,
    syllabi::SyllabusStore,
};

#[derive(Clone)]
pub struct AppState(Arc<Inner>);

pub struct Inner {
    pub catalog: Catalog,
    pub db: UserDb,
    /// `APP_URL` without a trailing slash: the only `Origin` writes are accepted from, and the base of absolute links.
    pub origin: String,
    /// Keycloak group whose members are admins; empty for none.
    pub admin_group: String,
    pub auth: AuthMode,
    pub syllabi: Option<SyllabusStore>,
    /// The built website, when `STATIC_DIR` is set.
    pub site: Option<Site>,
    /// Writes per signed-in user.
    pub writes: Limiter,
}

impl AppState {
    pub fn new(inner: Inner) -> Self {
        Self(Arc::new(inner))
    }
}

impl Deref for AppState {
    type Target = Inner;

    fn deref(&self) -> &Inner {
        &self.0
    }
}
