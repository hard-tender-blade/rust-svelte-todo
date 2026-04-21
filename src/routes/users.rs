use crate::{
    auth::{encode_jwt, try_authenticate},
    db,
    error::AppError,
    models::{CreateUser, Token, User},
};
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode},
};
use slug::slugify;
use sqlx::PgPool;
use ulid::Ulid;

#[utoipa::path(
    post,
    path = "/auth/signup",
    request_body = CreateUser,
    responses(
        (status = 201, description = "User created", body = Token),
        (status = 409, description = "Email already exists", body = crate::error::ErrorResponse),
    ),
    tag = "users"
)]
pub async fn signup(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateUser>,
) -> Result<(StatusCode, Json<Token>), AppError> {
    if db::users::get_by_email(&pool, &payload.email)
        .await?
        .is_some()
    {
        return Err(AppError::Conflict("email already in use".to_owned()));
    }

    let mut slug = slugify(&payload.full_name);
    for attempt in 0..3 {
        if db::users::get_by_slug(&pool, &slug).await?.is_some() {
            if attempt == 2 {
                return Err(AppError::Conflict(
                    "failed to generate unique slug after 3 attempts".to_owned(),
                ));
            }
            let suffix = &Ulid::new().to_string()[20..];
            slug = format!("{slug}-{suffix}");
        } else {
            break;
        }
    }

    let id = Ulid::new().to_string();
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(payload.password.as_bytes(), &salt)?
        .to_string();

    db::users::create(&pool, &id, &slug, &payload.full_name, &payload.email, &password_hash)
        .await?;

    Ok((StatusCode::CREATED, Json(Token { token: encode_jwt(&id)? })))
}

#[utoipa::path(
    post,
    path = "/auth/signin",
    request_body = CreateUser,
    responses(
        (status = 200, description = "User signed in", body = Token),
        (status = 401, description = "Invalid credentials", body = crate::error::ErrorResponse),
    ),
    tag = "users"
)]
pub async fn signin(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateUser>,
) -> Result<Json<Token>, AppError> {
    let user = db::users::get_by_email(&pool, &payload.email)
        .await?
        .ok_or(AppError::Unauthorized)?;

    let parsed_hash = PasswordHash::new(&user.password)?;
    if Argon2::default()
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .is_err()
    {
        return Err(AppError::Unauthorized);
    }

    Ok(Json(Token {
        token: encode_jwt(&user.id)?,
    }))
}

#[utoipa::path(
    get,
    path = "/users/me",
    responses(
        (status = 200, description = "Current user info", body = User),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse),
    ),
    tag = "users"
)]
pub async fn me(headers: HeaderMap, State(pool): State<PgPool>) -> Result<Json<User>, AppError> {
    let user = try_authenticate(&pool, &headers)
        .await
        .ok_or(AppError::Unauthorized)?;
    Ok(Json(user))
}
