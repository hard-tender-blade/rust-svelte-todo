use crate::{
    auth::try_authenticate,
    db,
    error::AppError,
    models::{CreateTodo, Todo, UpdateTodo},
};
use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
};
use slug::slugify;
use sqlx::PgPool;
use ulid::Ulid;

#[utoipa::path(
    get,
    path = "/todos",
    responses((status = 200, description = "List of todos", body = Vec<Todo>)),
    tag = "todos"
)]
pub async fn list_todos(
    headers: HeaderMap,
    State(pool): State<PgPool>,
) -> Result<Json<Vec<Todo>>, AppError> {
    try_authenticate(&pool, &headers)
        .await
        .ok_or(AppError::Unauthorized)?;

    let todos = db::todos::list(&pool).await?;

    Ok(Json(todos))
}

#[utoipa::path(
    post,
    path = "/todos",
    request_body = CreateTodo,
    responses(
        (status = 201, description = "Todo created", body = Todo),
        (status = 409, description = "Slug already exists", body = crate::error::ErrorResponse),
    ),
    tag = "todos"
)]
pub async fn create_todo(
    headers: HeaderMap,
    State(pool): State<PgPool>,
    Json(payload): Json<CreateTodo>,
) -> Result<(StatusCode, Json<Todo>), AppError> {
    try_authenticate(&pool, &headers)
        .await
        .ok_or(AppError::Unauthorized)?;

    let mut slug = slugify(&payload.title);
    for attempt in 0..3 {
        if db::todos::get_by_slug(&pool, &slug).await?.is_some() {
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

    let todo = db::todos::create(&pool, &slug, payload).await?;
    Ok((StatusCode::CREATED, Json(todo)))
}

#[utoipa::path(
    get,
    path = "/todos/{id}",
    responses(
        (status = 200, description = "Todo found", body = Todo),
        (status = 404, description = "Todo not found", body = crate::error::ErrorResponse),
    ),
    tag = "todos"
)]
pub async fn get_todo(
    headers: HeaderMap,
    State(pool): State<PgPool>,
    Path(id): Path<String>,
) -> Result<Json<Todo>, AppError> {
    try_authenticate(&pool, &headers)
        .await
        .ok_or(AppError::Unauthorized)?;

    let todo = db::todos::get_by_id(&pool, &id)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Json(todo))
}

#[utoipa::path(
    put,
    path = "/todos/{id}",
    request_body = UpdateTodo,
    responses(
        (status = 200, description = "Todo updated", body = Todo),
        (status = 404, description = "Todo not found", body = crate::error::ErrorResponse),
        (status = 409, description = "Slug already exists", body = crate::error::ErrorResponse),
    ),
    tag = "todos"
)]
pub async fn update_todo(
    headers: HeaderMap,
    State(pool): State<PgPool>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateTodo>,
) -> Result<Json<Todo>, AppError> {
    try_authenticate(&pool, &headers)
        .await
        .ok_or(AppError::Unauthorized)?;
    let mut slug = slugify(&payload.title);

    for attempt in 0..3 {
        if db::todos::get_by_slug(&pool, &slug).await?.is_some() {
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

    let todo = db::todos::update(&pool, &id, &slug, payload)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Json(todo))
}
