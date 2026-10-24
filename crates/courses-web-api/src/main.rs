//! CMU-authenticated personal storage and public/member SQLite course catalogs.
mod auth;
mod backend;

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result, anyhow};
use axum::{
    Extension, Router,
    error_handling::HandleErrorLayer,
    extract::FromRequestParts,
    http::request::Parts,
    response::{IntoResponse, Redirect},
    routing::get,
};
use axum_oidc::{
    AdditionalClaims, OidcAuthLayer, OidcClient, OidcLoginLayer, OidcSession,
    error::MiddlewareError,
    handle_oidc_redirect,
    openidconnect::{ClientId, ClientSecret, CsrfToken, IssuerUrl, Scope, core::CoreGenderClaim},
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::Serialize;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tower_sessions::{
    Expiry, MemoryStore, Session, SessionManagerLayer,
    cookie::{SameSite, time::Duration as CookieDuration},
};
use tracing::info;
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_scalar::{Scalar, Servable};

use auth::{AuthConfig, GroupClaims, OidcConfig};

/// Bridges tower-sessions to the axum-oidc session storage contract
struct SessionWrapper(Session);

impl<S: Send + Sync> FromRequestParts<S> for SessionWrapper {
    type Rejection = <Session as FromRequestParts<S>>::Rejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        Ok(Self(Session::from_request_parts(parts, state).await?))
    }
}

impl<AC: AdditionalClaims> axum_oidc::Session<AC> for SessionWrapper {
    type Error = tower_sessions::session::Error;

    async fn get(&self) -> Result<OidcSession<AC, CoreGenderClaim>, Self::Error> {
        Ok(self.0.get("axum-oidc").await?.unwrap_or_default())
    }

    async fn set(&mut self, value: OidcSession<AC, CoreGenderClaim>) -> Result<(), Self::Error> {
        self.0.insert("axum-oidc", value).await?;
        Ok(())
    }
}

/// OAuth2 state payload carrying the return_to and a CSRF token
#[derive(Serialize)]
struct RelayState<'a> {
    return_to: &'a str,
    csrf: uuid::Uuid,
}

/// Build the base64url state with a random csrf to guard against login CSRF
fn relay_state(return_to: &str) -> String {
    let state = RelayState {
        return_to,
        csrf: uuid::Uuid::new_v4(),
    };
    URL_SAFE_NO_PAD.encode(serde_json::to_vec(&state).expect("serialize relay state"))
}

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "health", description = "Health check"),
        (name = "catalog", description = "Course catalog"),
        (name = "auth", description = "Authentication"),
    ),
    info(title = "courses-web-api", version = "0.1.0")
)]
struct ApiDoc;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // devenv/secretspec injects these into the process environment. Read only
    // web API settings, without requiring unrelated scraper/CDN/build secrets.
    let env = |name: &str| std::env::var(name).with_context(|| format!("{name} must be set"));
    let port = std::env::var("PORT").unwrap_or_else(|_| "3002".into());
    let oidc_config = OidcConfig {
        keycloak_url: env("KEYCLOAK_URL")?,
        keycloak_realm: env("KEYCLOAK_REALM")?,
        client_id: env("OIDC_CLIENT_ID")?,
        client_secret: env("OIDC_CLIENT_SECRET")?,
        app_url: env("APP_URL")?,
        oauth_relay_url: env("OAUTH_RELAY_URL")?,
        project_admin_group: std::env::var("PROJECT_ADMIN_GROUP").unwrap_or_default(),
    };

    let app = build_app(oidc_config).await?;

    let addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&addr).await?;

    info!(
        addr = %addr,
        static_dir = ?std::env::var("STATIC_DIR").ok(),
        "listening"
    );
    axum::serve(listener, app).await?;
    Ok(())
}

async fn build_app(oidc_config: OidcConfig) -> Result<Router> {
    let backend = backend::Backend::open(
        std::env::var("CATALOG_DIR")
            .unwrap_or_else(|_| "data/catalog".into())
            .into(),
        std::env::var("USER_DB_PATH")
            .unwrap_or_else(|_| "data/private/users.sqlite".into())
            .into(),
        oidc_config.app_url.clone(),
    )?;
    let session_store = MemoryStore::default();
    let session_layer = ServiceBuilder::new().layer(
        SessionManagerLayer::new(session_store)
            .with_secure(oidc_config.app_url.starts_with("https://"))
            .with_same_site(SameSite::Lax)
            .with_expiry(Expiry::OnInactivity(CookieDuration::hours(1))),
    );

    let issuer_url = IssuerUrl::new(format!(
        "{}/realms/{}",
        oidc_config.keycloak_url, oidc_config.keycloak_realm
    ))
    .expect("valid issuer URL");

    let app_url = oidc_config.app_url.clone();
    let oidc_client = OidcClient::<GroupClaims>::builder()
        .with_default_http_client()
        .with_redirect_url(
            axum::http::Uri::try_from(oidc_config.oauth_relay_url.clone())
                .expect("valid OAUTH_RELAY_URL"),
        )
        .with_client_id(ClientId::new(oidc_config.client_id.clone()))
        .with_client_secret(ClientSecret::new(oidc_config.client_secret.clone()))
        .with_scopes(vec![
            Scope::new("openid".into()),
            Scope::new("email".into()),
            Scope::new("profile".into()),
        ])
        .with_state_generator(move || {
            CsrfToken::new(relay_state(&format!("{}/auth/callback", app_url)))
        })
        .discover(issuer_url)
        .await
        .map_err(|e| anyhow!("oidc discovery failed: {e}"))?
        .build();

    tracing::info!("OIDC discovery completed");

    let login_layer = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|e: MiddlewareError| async move {
            tracing::error!("OIDC login error: {:?}", e);
            e.into_response()
        }))
        .layer(OidcLoginLayer::<GroupClaims, SessionWrapper>::new());

    let auth_layer = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|e: MiddlewareError| async move {
            tracing::error!("OIDC auth error: {:?}", e);
            e.into_response()
        }))
        .layer(OidcAuthLayer::<GroupClaims, SessionWrapper>::new(
            oidc_client,
        ));

    let auth_config = Arc::new(AuthConfig {
        project_admin_group: oidc_config.project_admin_group,
    });

    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(login))
        .layer(login_layer)
        // Public catalog routes choose a sanitized database; private handlers require CMU identity.
        .route("/api/catalog/version", get(backend::version))
        .route("/api/catalog/binary", get(backend::binary))
        .route("/api/terms", get(backend::terms))
        .route("/api/courses", get(backend::courses))
        .route("/api/courses/{code}", get(backend::course))
        .route("/api/instructors", get(backend::instructors))
        .route("/api/instructors/{id}", get(backend::instructor))
        .route("/api/geneds", get(backend::geneds))
        .route("/api/programs", get(backend::programs))
        .route("/api/programs/{id}", get(backend::program))
        .route(
            "/api/schedules",
            get(backend::schedules).post(backend::create_schedule),
        )
        .route(
            "/api/schedules/{id}",
            axum::routing::put(backend::save_schedule).delete(backend::delete_schedule),
        )
        .route(
            "/api/schedules/{id}/share",
            axum::routing::post(backend::share).delete(backend::revoke_share),
        )
        .route("/api/shared-schedules/{id}", get(backend::shared))
        .route("/api/profile/courses", get(backend::saved))
        .route(
            "/api/profile/courses/{code}",
            axum::routing::put(backend::save_course).delete(backend::unsave_course),
        )
        .route(
            "/api/{*path}",
            get(|| async {
                (
                    axum::http::StatusCode::NOT_FOUND,
                    axum::Json(serde_json::json!({"message":"Endpoint unavailable"})),
                )
            }),
        )
        .routes(routes!(auth::me))
        .routes(routes!(health))
        .route(
            "/auth/callback",
            get(handle_oidc_redirect::<GroupClaims, SessionWrapper>),
        )
        .routes(routes!(logout))
        .layer(axum::middleware::from_fn(backend::guard))
        .layer(Extension(backend))
        .layer(axum::extract::DefaultBodyLimit::max(128 * 1024))
        .layer(auth_layer)
        .layer(session_layer)
        .layer(TraceLayer::new_for_http())
        .layer(Extension(auth_config))
        .split_for_parts();

    let router = router
        .merge(Scalar::with_url("/api/scalar", api.clone()))
        .route(
            "/api/openapi.json",
            get(move || {
                let api = api.clone();
                async move { axum::Json(api) }
            }),
        );

    // serve the built SPA bundle at / when STATIC_DIR is set
    let app = match std::env::var("STATIC_DIR") {
        Ok(dir) if !dir.is_empty() => {
            let dir = PathBuf::from(dir);
            let static_files = ServeDir::new(&dir)
                .append_index_html_on_directories(true)
                .fallback(ServeFile::new(dir.join("index.html")));
            router.fallback_service(static_files)
        }
        _ => router,
    };

    Ok(app)
}

#[utoipa::path(get, path = "/api/health", tag = "health", responses((status = OK, body = str)))]
async fn health() -> &'static str {
    "ok"
}

/// Login entry point that runs OIDC then returns to the app root
#[utoipa::path(get, path = "/auth/login", tag = "auth", responses((status = SEE_OTHER)))]
async fn login() -> Redirect {
    Redirect::to("/")
}

/// Clears the session and returns to the app root
#[utoipa::path(get, path = "/auth/logout", tag = "auth", responses((status = SEE_OTHER)))]
async fn logout(session: Session) -> Redirect {
    let _ = session.flush().await;
    Redirect::to("/")
}
