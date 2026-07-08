//! Web-app backend for the courses SPA. Serves the catalog binary and its
//! content hash under `/api/catalog/*`, the OpenAPI spec at
//! `/api/openapi.json`, and a Scalar reference at `/api/scalar`. When
//! `STATIC_DIR` is set the built SPA bundle is served at `/`.
//!
//! Every API route except `/api/health` is OIDC-protected: the `CurrentUser`
//! extractor returns `401` for anonymous requests, and `/auth/login` starts
//! the OIDC login flow.
//!
//! The catalog comes from one of two sources:
//!
//! - S3 mode by default. Credentials come from the typed secretspec SDK
//!   (`CDN_*`). The handler polls the object ETag and pulls the body into
//!   memory on change.
//! - Upstream-proxy mode when `CATALOG_UPSTREAM_URL` is set. The handler
//!   reverse-proxies `/api/catalog/*` to that URL, letting PR/staging deploys
//!   share the main deploy's catalog instead of getting their own scrape.

mod auth;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result, anyhow};
use arc_swap::ArcSwap;
use aws_sdk_s3::Client as S3Client;
use aws_sdk_s3::config::{BehaviorVersion, Credentials, Region};
use axum::{
    Extension, Router,
    error_handling::HandleErrorLayer,
    extract::{FromRequestParts, State},
    http::{StatusCode, header, request::Parts},
    response::{IntoResponse, Redirect, Response},
    routing::get,
};
use axum_oidc::{
    AdditionalClaims, OidcAuthLayer, OidcClient, OidcLoginLayer, OidcSession,
    error::MiddlewareError,
    handle_oidc_redirect,
    openidconnect::{ClientId, ClientSecret, CsrfToken, IssuerUrl, Scope, core::CoreGenderClaim},
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use bytes::Bytes;
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
use tracing::{info, warn};
use utoipa::{OpenApi, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_scalar::{Scalar, Servable};

use auth::{AuthConfig, CurrentUser, GroupClaims, OidcConfig};

secretspec_derive::declare_secrets!("../../secretspec.toml");

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

#[derive(Clone)]
enum CatalogSource {
    S3 {
        cache: Arc<ArcSwap<S3Cache>>,
    },
    Upstream {
        client: reqwest::Client,
        base: String,
    },
}

struct S3Cache {
    etag: String,
    bytes: Bytes,
}

#[derive(Clone)]
struct AppState {
    source: CatalogSource,
}

/// Identity of the currently-served catalog file
#[derive(Serialize, Clone, ToSchema)]
struct CatalogVersion {
    /// Opaque content identifier stable across re-fetches of identical content
    hash: String,
    /// File size in bytes
    bytes: u64,
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

/// Build an S3 client for the CDN bucket from the given endpoint and credentials.
fn build_s3_client(endpoint: String, access_key_id: String, secret_access_key: String) -> S3Client {
    let credentials = Credentials::new(access_key_id, secret_access_key, None, None, "courses-cdn");
    let config = aws_sdk_s3::config::Builder::new()
        .behavior_version(BehaviorVersion::latest())
        .region(Region::new("us-east-1"))
        .endpoint_url(endpoint)
        .credentials_provider(credentials)
        .force_path_style(true)
        .build();
    S3Client::from_conf(config)
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let secrets = SecretSpec::builder()
        .with_provider("env")
        .with_reason("courses-web-api startup")
        .load()?
        .secrets;

    let port = std::env::var("PORT").unwrap_or_else(|_| "3002".into());
    let s3_key = std::env::var("S3_KEY").unwrap_or_else(|_| "catalog.bin".into());
    let poll_interval: u64 = std::env::var("POLL_INTERVAL")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(300);

    let oidc_config = OidcConfig {
        keycloak_url: secrets.keycloak_url.context("KEYCLOAK_URL must be set")?,
        keycloak_realm: secrets
            .keycloak_realm
            .context("KEYCLOAK_REALM must be set")?,
        client_id: secrets
            .oidc_client_id
            .context("OIDC_CLIENT_ID must be set")?,
        client_secret: secrets
            .oidc_client_secret
            .context("OIDC_CLIENT_SECRET must be set")?,
        app_url: std::env::var("APP_URL").context("APP_URL must be set")?,
        oauth_relay_url: secrets
            .oauth_relay_url
            .context("OAUTH_RELAY_URL must be set")?,
        project_group: secrets.project_group.context("PROJECT_GROUP must be set")?,
        project_admin_group: secrets
            .project_admin_group
            .context("PROJECT_ADMIN_GROUP must be set")?,
    };

    // upstream proxy when CATALOG_UPSTREAM_URL is set, else S3 from CDN_*
    let source = match std::env::var("CATALOG_UPSTREAM_URL") {
        Ok(url) if !url.is_empty() => {
            let base = url.trim_end_matches('/').to_string();
            info!(upstream = %base, "running in upstream-proxy mode");
            CatalogSource::Upstream {
                client: reqwest::Client::new(),
                base,
            }
        }
        _ => {
            let endpoint = secrets
                .cdn_s3_endpoint
                .context("CDN_S3_ENDPOINT must be set")?;
            let bucket = secrets.cdn_s3_bucket.context("CDN_S3_BUCKET must be set")?;
            let access_key_id = secrets
                .cdn_access_key_id
                .context("CDN_ACCESS_KEY_ID must be set")?;
            let secret_access_key = secrets
                .cdn_secret_access_key
                .context("CDN_SECRET_ACCESS_KEY must be set")?;
            let client = build_s3_client(endpoint, access_key_id, secret_access_key);

            let initial = fetch_object(&client, &bucket, &s3_key).await?;
            info!(
                bucket = %bucket,
                key = %s3_key,
                etag = %initial.etag,
                bytes = initial.bytes.len(),
                "initial catalog fetched"
            );
            let cache = Arc::new(ArcSwap::from_pointee(initial));
            spawn_s3_poller(
                client.clone(),
                bucket.clone(),
                s3_key.clone(),
                Duration::from_secs(poll_interval),
                cache.clone(),
            );
            CatalogSource::S3 { cache }
        }
    };

    let state = AppState { source };
    let app = build_app(state, oidc_config).await?;

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

async fn build_app(state: AppState, oidc_config: OidcConfig) -> Result<Router> {
    let session_store = MemoryStore::default();
    let session_layer = ServiceBuilder::new().layer(
        SessionManagerLayer::new(session_store)
            .with_secure(false)
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
        project_group: oidc_config.project_group,
        project_admin_group: oidc_config.project_admin_group,
    });

    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(login))
        .layer(login_layer)
        // protected catalog + identity routes guarded by the CurrentUser extractor
        .routes(routes!(catalog_version))
        .routes(routes!(catalog_binary))
        .routes(routes!(auth::me))
        .routes(routes!(health))
        .route(
            "/auth/callback",
            get(handle_oidc_redirect::<GroupClaims, SessionWrapper>),
        )
        .routes(routes!(logout))
        .layer(auth_layer)
        .layer(session_layer)
        .layer(TraceLayer::new_for_http())
        .layer(Extension(auth_config))
        .with_state(state)
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

async fn fetch_object(client: &S3Client, bucket: &str, key: &str) -> Result<S3Cache> {
    let resp = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .send()
        .await
        .with_context(|| format!("get_object s3://{bucket}/{key}"))?;
    let etag = resp
        .e_tag()
        .ok_or_else(|| anyhow!("missing ETag on s3://{bucket}/{key}"))?
        .trim_matches('"')
        .to_string();
    let bytes = resp.body.collect().await?.into_bytes();
    Ok(S3Cache { etag, bytes })
}

async fn fetch_etag(client: &S3Client, bucket: &str, key: &str) -> Result<String> {
    let resp = client
        .head_object()
        .bucket(bucket)
        .key(key)
        .send()
        .await
        .with_context(|| format!("head_object s3://{bucket}/{key}"))?;
    Ok(resp
        .e_tag()
        .ok_or_else(|| anyhow!("missing ETag on s3://{bucket}/{key}"))?
        .trim_matches('"')
        .to_string())
}

fn spawn_s3_poller(
    client: S3Client,
    bucket: String,
    key: String,
    interval: Duration,
    cache: Arc<ArcSwap<S3Cache>>,
) {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(interval).await;
            let current_etag = cache.load().etag.clone();
            match fetch_etag(&client, &bucket, &key).await {
                Ok(remote_etag) if remote_etag != current_etag => {
                    match fetch_object(&client, &bucket, &key).await {
                        Ok(next) => {
                            info!(
                                etag = %next.etag,
                                bytes = next.bytes.len(),
                                "catalog refreshed"
                            );
                            cache.store(Arc::new(next));
                        }
                        Err(e) => warn!(error = %e, "refetch failed"),
                    }
                }
                Ok(_) => {}
                Err(e) => warn!(error = %e, "etag check failed"),
            }
        }
    });
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

#[utoipa::path(
    get,
    path = "/api/catalog/version",
    tag = "catalog",
    responses((status = OK, body = CatalogVersion), (status = UNAUTHORIZED))
)]
async fn catalog_version(_user: CurrentUser, State(state): State<AppState>) -> Response {
    match &state.source {
        CatalogSource::S3 { cache } => {
            let cur = cache.load_full();
            axum::Json(CatalogVersion {
                hash: cur.etag.clone(),
                bytes: cur.bytes.len() as u64,
            })
            .into_response()
        }
        CatalogSource::Upstream { client, base } => {
            match client
                .get(format!("{base}/api/catalog/version"))
                .send()
                .await
            {
                Ok(resp) => {
                    let status =
                        StatusCode::from_u16(resp.status().as_u16()).unwrap_or(StatusCode::OK);
                    let bytes = resp.bytes().await.unwrap_or_default();
                    let mut response = Response::new(axum::body::Body::from(bytes));
                    *response.status_mut() = status;
                    response
                        .headers_mut()
                        .insert(header::CONTENT_TYPE, "application/json".parse().unwrap());
                    response
                }
                Err(e) => (StatusCode::BAD_GATEWAY, format!("upstream: {e}")).into_response(),
            }
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/catalog/binary",
    tag = "catalog",
    responses(
        (status = OK, description = "Catalog binary, gzipped (Content-Encoding: gzip)", content_type = "application/octet-stream"),
        (status = UNAUTHORIZED),
        (status = BAD_GATEWAY, description = "Upstream proxy failure")
    )
)]
async fn catalog_binary(_user: CurrentUser, State(state): State<AppState>) -> Response {
    match &state.source {
        CatalogSource::S3 { cache } => {
            let cur = cache.load_full();
            let is_gzip = cur.bytes.starts_with(&[0x1f, 0x8b]);
            let mut response = Response::new(axum::body::Body::from(cur.bytes.clone()));
            let h = response.headers_mut();
            h.insert(
                header::CONTENT_TYPE,
                "application/octet-stream".parse().unwrap(),
            );
            h.insert(header::CONTENT_LENGTH, cur.bytes.len().into());
            h.insert(header::CACHE_CONTROL, "public, max-age=60".parse().unwrap());
            if is_gzip {
                h.insert(header::CONTENT_ENCODING, "gzip".parse().unwrap());
            }
            response
        }
        CatalogSource::Upstream { client, base } => {
            match client
                .get(format!("{base}/api/catalog/binary"))
                .send()
                .await
            {
                Ok(resp) => {
                    let status =
                        StatusCode::from_u16(resp.status().as_u16()).unwrap_or(StatusCode::OK);
                    let mut response =
                        Response::new(axum::body::Body::from_stream(resp.bytes_stream()));
                    *response.status_mut() = status;
                    response.headers_mut().insert(
                        header::CONTENT_TYPE,
                        "application/octet-stream".parse().unwrap(),
                    );
                    response
                        .headers_mut()
                        .insert(header::CONTENT_ENCODING, "gzip".parse().unwrap());
                    response
                }
                Err(e) => (StatusCode::BAD_GATEWAY, format!("upstream: {e}")).into_response(),
            }
        }
    }
}
