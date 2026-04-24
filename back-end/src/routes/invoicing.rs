use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use ulid::Ulid;

use crate::{
    auth::OptionalAuthSession,
    db::DatabaseService,
    error::AppError,
    models::{InvoicingEntry, UpsertInvoicingEntry},
};

#[derive(Deserialize, utoipa::IntoParams)]
pub struct MonthQuery {
    /// Calendar year, e.g. `2025`.
    pub year: i32,
    /// Calendar month (1–12).
    pub month: u32,
}

/// List all invoicing entries
///
/// Returns all invoicing entries across all tenants, ordered by date descending.
#[utoipa::path(
    get,
    path = "/admin/invoicing",
    params(
        ("Authorization" = String, Header, description = "Bearer access token. Format: `Bearer <token>`")
    ),
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "List of all invoicing entries", body = Vec<InvoicingEntry>),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse),
    ),
    tag = "Admin Invoicing"
)]
#[tracing::instrument(skip_all)]
pub async fn list_all_invoicing_entries(
    OptionalAuthSession(user): OptionalAuthSession,
    State(db): State<DatabaseService>,
) -> Result<Json<Vec<InvoicingEntry>>, AppError> {
    user.ok_or(AppError::Unauthorized)?;

    let entries = db.list_all_invoicing_entries().await?;
    Ok(Json(entries))
}

/// List invoicing entries for a tenant
///
/// Returns all invoicing entries for the specified tenant, ordered by date descending.
#[utoipa::path(
    get,
    path = "/admin/invoicing/{mongo_id}",
    params(
        ("mongo_id" = String, Path, description = "MongoDB ObjectId of the tenant"),
        ("Authorization" = String, Header, description = "Bearer access token. Format: `Bearer <token>`")
    ),
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "List of invoicing entries", body = Vec<InvoicingEntry>),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse),
    ),
    tag = "Admin Invoicing"
)]
#[tracing::instrument(skip_all, fields(tenant.mongo_id = %mongo_id))]
pub async fn list_invoicing_entries(
    OptionalAuthSession(user): OptionalAuthSession,
    State(db): State<DatabaseService>,
    Path(mongo_id): Path<String>,
) -> Result<Json<Vec<InvoicingEntry>>, AppError> {
    user.ok_or(AppError::Unauthorized)?;

    let entries = db.list_invoicing_entries(&mongo_id).await?;
    Ok(Json(entries))
}

/// List invoicing entries for a tenant within a calendar month
///
/// Returns all invoicing entries for the specified tenant that fall within the given
/// year/month, ordered by date ascending.
#[utoipa::path(
    get,
    path = "/admin/invoicing/{mongo_id}/month",
    params(
        ("mongo_id" = String, Path, description = "MongoDB ObjectId of the tenant"),
        MonthQuery,
        ("Authorization" = String, Header, description = "Bearer access token. Format: `Bearer <token>`")
    ),
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Invoicing entries for the month", body = Vec<InvoicingEntry>),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse),
    ),
    tag = "Admin Invoicing"
)]
#[tracing::instrument(skip_all, fields(tenant.mongo_id = %mongo_id, query.year = %query.year, query.month = %query.month))]
pub async fn list_invoicing_entries_for_month(
    OptionalAuthSession(user): OptionalAuthSession,
    State(db): State<DatabaseService>,
    Path(mongo_id): Path<String>,
    Query(query): Query<MonthQuery>,
) -> Result<Json<Vec<InvoicingEntry>>, AppError> {
    user.ok_or(AppError::Unauthorized)?;

    let entries = db
        .list_invoicing_entries_for_month(&mongo_id, query.year, query.month)
        .await?;
    Ok(Json(entries))
}

/// Get a single invoicing entry
#[utoipa::path(
    get,
    path = "/admin/invoicing/entry/{id}",
    params(
        ("id" = String, Path, description = "ULID of the invoicing entry"),
        ("Authorization" = String, Header, description = "Bearer access token. Format: `Bearer <token>`")
    ),
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Invoicing entry found", body = InvoicingEntry),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse),
        (status = 404, description = "Invoicing entry not found", body = crate::error::ErrorResponse),
    ),
    tag = "Admin Invoicing"
)]
#[tracing::instrument(skip_all, fields(invoicing.id = %id))]
pub async fn get_invoicing_entry(
    OptionalAuthSession(user): OptionalAuthSession,
    State(db): State<DatabaseService>,
    Path(id): Path<String>,
) -> Result<Json<InvoicingEntry>, AppError> {
    user.ok_or(AppError::Unauthorized)?;

    let entry = db.get_invoicing_entry(&id).await?.ok_or_else(|| {
        tracing::warn!(invoicing.id = %id, "invoicing entry not found");
        AppError::NotFound
    })?;

    Ok(Json(entry))
}

/// Create or update an invoicing entry
///
/// Upserts an invoicing entry. When `id` is omitted in the path a new ULID is generated
/// (create). When an existing `id` is supplied the record is updated in place.
/// This single endpoint handles both create and update.
#[utoipa::path(
    put,
    path = "/admin/invoicing/entry/{id}",
    request_body = UpsertInvoicingEntry,
    params(
        ("id" = String, Path, description = "ULID of the invoicing entry (generate a new ULID client-side for creates)"),
        ("Authorization" = String, Header, description = "Bearer access token. Format: `Bearer <token>`")
    ),
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Invoicing entry upserted", body = InvoicingEntry),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse),
    ),
    tag = "Admin Invoicing"
)]
#[tracing::instrument(skip_all, fields(invoicing.id = %id))]
pub async fn upsert_invoicing_entry(
    OptionalAuthSession(user): OptionalAuthSession,
    State(db): State<DatabaseService>,
    Path(id): Path<String>,
    Json(payload): Json<UpsertInvoicingEntry>,
) -> Result<Json<InvoicingEntry>, AppError> {
    user.ok_or(AppError::Unauthorized)?;

    let entry = db
        .upsert_invoicing_entry(
            &id,
            &payload.mongo_id,
            payload.date,
            payload.price,
            payload.note.as_deref(),
        )
        .await?;

    tracing::info!(invoicing.id = %entry.id, tenant.mongo_id = %entry.mongo_id, "invoicing entry upserted");
    Ok(Json(entry))
}

/// Create a new invoicing entry (server-generated id)
///
/// Convenience POST endpoint that generates a ULID server-side and inserts the entry.
#[utoipa::path(
    post,
    path = "/admin/invoicing/entry",
    request_body = UpsertInvoicingEntry,
    params(
        ("Authorization" = String, Header, description = "Bearer access token. Format: `Bearer <token>`")
    ),
    security(("bearerAuth" = [])),
    responses(
        (status = 201, description = "Invoicing entry created", body = InvoicingEntry),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse),
    ),
    tag = "Admin Invoicing"
)]
#[tracing::instrument(skip_all)]
pub async fn create_invoicing_entry(
    OptionalAuthSession(user): OptionalAuthSession,
    State(db): State<DatabaseService>,
    Json(payload): Json<UpsertInvoicingEntry>,
) -> Result<(StatusCode, Json<InvoicingEntry>), AppError> {
    user.ok_or(AppError::Unauthorized)?;

    let id = Ulid::new().to_string();
    let entry = db
        .upsert_invoicing_entry(
            &id,
            &payload.mongo_id,
            payload.date,
            payload.price,
            payload.note.as_deref(),
        )
        .await?;

    tracing::info!(invoicing.id = %entry.id, tenant.mongo_id = %entry.mongo_id, "invoicing entry created");
    Ok((StatusCode::CREATED, Json(entry)))
}

/// Delete an invoicing entry
#[utoipa::path(
    delete,
    path = "/admin/invoicing/entry/{id}",
    params(
        ("id" = String, Path, description = "ULID of the invoicing entry"),
        ("Authorization" = String, Header, description = "Bearer access token. Format: `Bearer <token>`")
    ),
    security(("bearerAuth" = [])),
    responses(
        (status = 204, description = "Invoicing entry deleted"),
        (status = 401, description = "Unauthorized", body = crate::error::ErrorResponse),
        (status = 404, description = "Invoicing entry not found", body = crate::error::ErrorResponse),
    ),
    tag = "Admin Invoicing"
)]
#[tracing::instrument(skip_all, fields(invoicing.id = %id))]
pub async fn delete_invoicing_entry(
    OptionalAuthSession(user): OptionalAuthSession,
    State(db): State<DatabaseService>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    user.ok_or(AppError::Unauthorized)?;

    let deleted = db.delete_invoicing_entry(&id).await?;
    if !deleted {
        tracing::warn!(invoicing.id = %id, "invoicing entry not found for delete");
        return Err(AppError::NotFound);
    }

    tracing::info!(invoicing.id = %id, "invoicing entry deleted");
    Ok(StatusCode::NO_CONTENT)
}
