use axum::http::HeaderMap;
use sqlx::PgPool;
use crate::{db, error::AppError, models::{Claims, User}};

pub async fn try_authenticate(pool: &PgPool, headers: &HeaderMap) -> Option<User> {
    let token = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))?;

    let jwt_secret =
        std::env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret-change-me".to_string());

    let mut validation = jsonwebtoken::Validation::default();
    validation.validate_exp = false;
    validation.required_spec_claims = std::collections::HashSet::new();

    let claims = jsonwebtoken::decode::<Claims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(jwt_secret.as_bytes()),
        &validation,
    )
    .ok()?
    .claims;

    db::users::get_by_id(pool, &claims.id)
        .await
        .ok()
        .flatten()
}

pub fn encode_jwt(id: &str) -> Result<String, AppError> {
    let jwt_secret =
        std::env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret-change-me".to_string());
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &Claims { id: id.to_owned() },
        &jsonwebtoken::EncodingKey::from_secret(jwt_secret.as_bytes()),
    )?;
    Ok(token)
}
