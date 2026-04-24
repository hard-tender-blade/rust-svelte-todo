use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use ulid::Ulid;

use crate::{
    auth::OptionalAuthSession,
    db::DatabaseService,
    error::AppError,
    models::{OnboardingEntry, UpsertOnboardingEntry},
    mongo::{self, MongoService},
};

/// List all onboarding entries
///
/// Returns all onboarding entries across all tenants, ordered by created_at descending.
#[utoipa::path(
    get,
    path = "/admin/onboarding",
    params(
        ("Authorization" = String, Header, description = "Bearer access token. Format: `Bearer <token>`")
    ),
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "List of all onboarding entries", body = Vec<OnboardingEntry>),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse),
    ),
    tag = "Admin Onboarding"
)]
#[tracing::instrument(skip_all)]
pub async fn list_all_onboarding_entries(
    OptionalAuthSession(user): OptionalAuthSession,
    State(db): State<DatabaseService>,
) -> Result<Json<Vec<OnboardingEntry>>, AppError> {
    user.ok_or(AppError::Unauthorized)?;

    let entries = db.list_all_onboarding_entries().await?;
    Ok(Json(entries))
}

/// List onboarding entries for a tenant
///
/// Returns all onboarding entries for the specified tenant, ordered by created_at descending.
#[utoipa::path(
    get,
    path = "/admin/onboarding/{mongo_id}",
    params(
        ("mongo_id" = String, Path, description = "MongoDB ObjectId of the tenant"),
        ("Authorization" = String, Header, description = "Bearer access token. Format: `Bearer <token>`")
    ),
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "List of onboarding entries", body = Vec<OnboardingEntry>),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse),
    ),
    tag = "Admin Onboarding"
)]
#[tracing::instrument(skip_all, fields(tenant.mongo_id = %mongo_id))]
pub async fn list_onboarding_entries(
    OptionalAuthSession(user): OptionalAuthSession,
    State(db): State<DatabaseService>,
    Path(mongo_id): Path<String>,
) -> Result<Json<Vec<OnboardingEntry>>, AppError> {
    user.ok_or(AppError::Unauthorized)?;

    let entries = db.list_onboarding_entries(&mongo_id).await?;
    Ok(Json(entries))
}

/// Get a single onboarding entry
#[utoipa::path(
    get,
    path = "/admin/onboarding/entry/{id}",
    params(
        ("id" = String, Path, description = "ULID of the onboarding entry"),
        ("Authorization" = String, Header, description = "Bearer access token. Format: `Bearer <token>`")
    ),
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Onboarding entry found", body = OnboardingEntry),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse),
        (status = 404, description = "Onboarding entry not found", body = crate::error::ErrorResponse),
    ),
    tag = "Admin Onboarding"
)]
#[tracing::instrument(skip_all, fields(onboarding.id = %id))]
pub async fn get_onboarding_entry(
    OptionalAuthSession(user): OptionalAuthSession,
    State(db): State<DatabaseService>,
    Path(id): Path<String>,
) -> Result<Json<OnboardingEntry>, AppError> {
    user.ok_or(AppError::Unauthorized)?;

    let entry = db.get_onboarding_entry(&id).await?.ok_or_else(|| {
        tracing::warn!(onboarding.id = %id, "onboarding entry not found");
        AppError::NotFound
    })?;

    Ok(Json(entry))
}

/// Create a new onboarding entry (server-generated id)
#[utoipa::path(
    post,
    path = "/admin/onboarding/entry",
    request_body = UpsertOnboardingEntry,
    params(
        ("Authorization" = String, Header, description = "Bearer access token. Format: `Bearer <token>`")
    ),
    security(("bearerAuth" = [])),
    responses(
        (status = 201, description = "Onboarding entry created", body = OnboardingEntry),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse),
    ),
    tag = "Admin Onboarding"
)]
#[tracing::instrument(skip_all)]
pub async fn create_onboarding_entry(
    OptionalAuthSession(user): OptionalAuthSession,
    State(db): State<DatabaseService>,
    State(mongo): State<MongoService>,
    Json(payload): Json<UpsertOnboardingEntry>,
) -> Result<(StatusCode, Json<OnboardingEntry>), AppError> {
    user.ok_or(AppError::Unauthorized)?;

    mongo.get_tenant(&payload.mongo_id).await?;

    let id = Ulid::new().to_string();
    let entry = db
        .upsert_onboarding_entry(
            &id,
            &payload.mongo_id,
            payload.date_training,
            payload.paid,
            payload.price,
            &payload.currency,
            payload.invoiced,
            payload.invoiced_date,
            payload.business_module,
            payload.fans_module,
            payload.note.as_deref(),
            payload.enigoo_involved,
        )
        .await?;

    tracing::info!(onboarding.id = %entry.id, tenant.mongo_id = %entry.mongo_id, "onboarding entry created");
    Ok((StatusCode::CREATED, Json(entry)))
}

/// Create or update an onboarding entry
///
/// Upserts an onboarding entry by id.
#[utoipa::path(
    put,
    path = "/admin/onboarding/entry/{id}",
    request_body = UpsertOnboardingEntry,
    params(
        ("id" = String, Path, description = "ULID of the onboarding entry (generate a new ULID client-side for creates)"),
        ("Authorization" = String, Header, description = "Bearer access token. Format: `Bearer <token>`")
    ),
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Onboarding entry upserted", body = OnboardingEntry),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse),
    ),
    tag = "Admin Onboarding"
)]
#[tracing::instrument(skip_all, fields(onboarding.id = %id))]
pub async fn upsert_onboarding_entry(
    OptionalAuthSession(user): OptionalAuthSession,
    State(db): State<DatabaseService>,
    Path(id): Path<String>,
    Json(payload): Json<UpsertOnboardingEntry>,
) -> Result<Json<OnboardingEntry>, AppError> {
    user.ok_or(AppError::Unauthorized)?;

    let entry = db
        .upsert_onboarding_entry(
            &id,
            &payload.mongo_id,
            payload.date_training,
            payload.paid,
            payload.price,
            &payload.currency,
            payload.invoiced,
            payload.invoiced_date,
            payload.business_module,
            payload.fans_module,
            payload.note.as_deref(),
            payload.enigoo_involved,
        )
        .await?;

    tracing::info!(onboarding.id = %entry.id, tenant.mongo_id = %entry.mongo_id, "onboarding entry upserted");
    Ok(Json(entry))
}

/// Delete an onboarding entry
#[utoipa::path(
    delete,
    path = "/admin/onboarding/entry/{id}",
    params(
        ("id" = String, Path, description = "ULID of the onboarding entry"),
        ("Authorization" = String, Header, description = "Bearer access token. Format: `Bearer <token>`")
    ),
    security(("bearerAuth" = [])),
    responses(
        (status = 204, description = "Onboarding entry deleted"),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse),
        (status = 404, description = "Onboarding entry not found", body = crate::error::ErrorResponse),
    ),
    tag = "Admin Onboarding"
)]
#[tracing::instrument(skip_all, fields(onboarding.id = %id))]
pub async fn delete_onboarding_entry(
    OptionalAuthSession(user): OptionalAuthSession,
    State(db): State<DatabaseService>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    user.ok_or(AppError::Unauthorized)?;

    let deleted = db.delete_onboarding_entry(&id).await?;
    if !deleted {
        tracing::warn!(onboarding.id = %id, "onboarding entry not found for delete");
        return Err(AppError::NotFound);
    }

    tracing::info!(onboarding.id = %id, "onboarding entry deleted");
    Ok(StatusCode::NO_CONTENT)
}
