//! OIDC authentication: verified CMU email claims establish membership, and
//! the `CurrentUser` extractor turns a valid session into an authenticated
//! principal (or a `401`).

use axum::{
    Extension, Json,
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
};
use axum_oidc::{AdditionalClaims, OidcClaims, openidconnect};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub struct OidcConfig {
    pub keycloak_url: String,
    pub keycloak_realm: String,
    pub client_id: String,
    pub client_secret: String,
    pub app_url: String,
    pub oauth_relay_url: String,
    pub project_admin_group: String,
}

/// Group path granting administrative capabilities
pub struct AuthConfig {
    pub project_admin_group: String,
}

/// Group memberships carried on the OIDC token as full paths
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupClaims {
    #[serde(default)]
    pub groups: Vec<String>,
}

impl openidconnect::AdditionalClaims for GroupClaims {}
impl AdditionalClaims for GroupClaims {}

/// Authenticated user extracted from OIDC claims
pub struct CurrentUser {
    pub subject: String,
    pub name: String,
    pub groups: Vec<String>,
    // Reserved for profile implementations.
    #[allow(dead_code)]
    pub andrew_id: String,
}

impl CurrentUser {
    pub fn is_admin(&self, cfg: &AuthConfig) -> bool {
        self.groups.contains(&cfg.project_admin_group)
    }
}

impl<S> FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let claims = OidcClaims::<GroupClaims>::from_request_parts(parts, state)
            .await
            .map_err(|_| StatusCode::UNAUTHORIZED)?;

        // Only a verified university email from the configured, verified OIDC issuer
        // is accepted. Project groups alone do not establish CMU affiliation.
        let cmu_email = verified_cmu_email(
            claims.email().map(|email| email.as_str()),
            claims.email_verified() == Some(true),
        );
        if !cmu_email {
            return Err(StatusCode::FORBIDDEN);
        }
        let subject = claims.subject().to_string();
        let name = claims
            .name()
            .and_then(|n| n.get(None))
            .map(|n| n.as_str().to_owned())
            .or_else(|| claims.preferred_username().map(|u| u.as_str().to_owned()))
            .unwrap_or_else(|| subject.clone());
        let groups = claims.additional_claims().groups.clone();

        let andrew_id = claims
            .email()
            .unwrap()
            .as_str()
            .split('@')
            .next()
            .unwrap()
            .to_ascii_lowercase();
        Ok(CurrentUser {
            andrew_id,
            subject,
            name,
            groups,
        })
    }
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct UserInfo {
    pub subject: String,
    pub name: String,
    pub can_create: bool,
    pub is_admin: bool,
}

#[utoipa::path(
    get,
    path = "/api/me",
    tag = "auth",
    responses((status = OK, body = UserInfo), (status = UNAUTHORIZED))
)]
pub async fn me(
    user: CurrentUser,
    Extension(auth): Extension<Arc<AuthConfig>>,
) -> Result<Json<UserInfo>, crate::backend::ApiError> {
    Ok(Json(UserInfo {
        can_create: true,
        is_admin: user.is_admin(&auth),
        name: user.name,
        subject: user.subject,
    }))
}

fn verified_cmu_email(email: Option<&str>, verified: bool) -> bool {
    verified
        && email.is_some_and(|email| {
            email
                .to_ascii_lowercase()
                .rsplit_once('@')
                .is_some_and(|(local, domain)| {
                    !local.is_empty() && (domain == "cmu.edu" || domain.ends_with(".cmu.edu"))
                })
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn university_identity_requires_verified_exact_domain() {
        assert!(verified_cmu_email(Some("user@andrew.cmu.edu"), true));
        assert!(verified_cmu_email(Some("user@CMU.EDU"), true));
        assert!(!verified_cmu_email(Some("user@cmu.edu.evil.test"), true));
        assert!(!verified_cmu_email(Some("user@notcmu.edu"), true));
        assert!(!verified_cmu_email(Some("user@cmu.edu"), false));
        assert!(!verified_cmu_email(None, true));
    }
}
