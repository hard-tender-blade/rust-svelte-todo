use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use slug::slugify;
use sqlx::PgPool;
use ulid::Ulid;
use utoipa::{OpenApi, ToSchema};
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_scalar::{Scalar, Servable};
use utoipa_swagger_ui::SwaggerUi;

#[derive(Serialize, Deserialize, ToSchema, Clone, sqlx::FromRow)]
struct Todo {
    id: String,
    slug: String,
    title: String,
    description: String,
    completed: bool,
}

#[derive(Deserialize, ToSchema)]
struct CreateTodo {
    title: String,
    description: String,
}

#[derive(Deserialize, ToSchema)]
struct UpdateTodo {
    title: String,
    description: String,
    completed: bool,
}

// The JSON body sent on every error response.
#[derive(Serialize, ToSchema)]
struct ErrorResponse {
    error: String,
}

impl ErrorResponse {
    fn new(msg: impl Into<String>) -> Json<Self> {
        Json(Self { error: msg.into() })
    }
}

// Every variant maps to a specific HTTP status + message.
// Adding a new error case is just adding a variant here.
enum AppError {
    NotFound,
    Conflict(String),      // e.g. unique constraint — caller provides the message
    Internal(sqlx::Error), // unexpected DB error — message is hidden from the client
}

// IntoResponse is the axum trait that lets a type be returned from a handler.
// Implementing it here means handlers can return Result<T, AppError> and axum
// knows how to turn the error into an HTTP response automatically.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            Self::NotFound => {
                (StatusCode::NOT_FOUND, ErrorResponse::new("not found")).into_response()
            }
            Self::Conflict(msg) => (StatusCode::CONFLICT, ErrorResponse::new(msg)).into_response(),
            Self::Internal(err) => {
                // Log the real error server-side, never leak internals to the client.
                eprintln!("internal error: {err}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorResponse::new("internal server error"),
                )
                    .into_response()
            }
        }
    }
}

// From<sqlx::Error> lets us use ? in handlers — sqlx errors are automatically
// converted into the right AppError variant.
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        match &err {
            sqlx::Error::Database(db_err) => {
                match db_err.code().as_deref() {
                    // Postgres error code 23505 = unique_violation
                    Some("23505") => Self::Conflict(db_err.constraint().map_or_else(
                        || "duplicate value".to_owned(),
                        |c| format!("duplicate value violates unique constraint '{c}'"),
                    )),
                    _ => Self::Internal(err),
                }
            }
            _ => Self::Internal(err),
        }
    }
}

#[derive(OpenApi)]
#[openapi(tags((name = "todos", description = "Todo management")))]
struct ApiDoc;

type AppState = PgPool;

#[tokio::main]
async fn main() {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(list_todos, create_todo))
        .routes(routes!(get_todo, update_todo))
        .with_state(pool)
        .split_for_parts();

    let app = router
        .merge(SwaggerUi::new("/swagger").url("/openapi.json", api.clone()))
        .merge(Scalar::with_url("/scalar", api.clone()));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on http://localhost:3000");
    println!("Swagger UI at http://localhost:3000/swagger/");
    println!("Scalar UI at http://localhost:3000/scalar/");
    println!("OpenAPI JSON at http://localhost:3000/openapi.json");
    axum::serve(listener, app).await.unwrap();
}

#[utoipa::path(
    get,
    path = "/todos",
    responses((status = 200, description = "List of todos", body = Vec<Todo>)),
    tag = "todos"
)]
async fn list_todos(State(pool): State<AppState>) -> Result<Json<Vec<Todo>>, AppError> {
    let todos = sqlx::query_as::<_, Todo>("SELECT * FROM todos")
        .fetch_all(&pool)
        .await?; // ? calls From<sqlx::Error> for AppError, then returns early if Err
    Ok(Json(todos))
}

#[utoipa::path(
    post,
    path = "/todos",
    request_body = CreateTodo,
    responses(
        (status = 201, description = "Todo created", body = Todo),
        (status = 409, description = "Slug already exists", body = ErrorResponse),
    ),
    tag = "todos"
)]
async fn create_todo(
    State(pool): State<AppState>,
    Json(payload): Json<CreateTodo>,
) -> Result<(StatusCode, Json<Todo>), AppError> {
    let id = Ulid::new().to_string();
    let mut slug = slugify(&payload.title);

    // check slog uniqueness before attempting insert and add ulid to the slug to guarantee uniqueness without relying on DB errors for control flow
    for attempt in 0..3 {
        if let Some(existing) = get_todo_by_slug(State(pool.clone()), slug.clone()).await? {
            if attempt == 2 {
                return Err(AppError::Conflict(format!(
                    "Failed to generate unique slug after 3 attempts"
                )));
            }
            // append a short random string to the slug and try again
            let random_suffix: String = Ulid::new().to_string()[20..].to_string();
            let new_slug = format!("{}-{}", slug, random_suffix);
            println!("Slug '{slug}' already exists, trying '{new_slug}'");
            slug = new_slug;
        } else {
            break; // slug is unique, proceed with insert
        }
    }

    let todo = sqlx::query_as::<_, Todo>(
        "INSERT INTO todos (id, slug, title, description) VALUES ($1, $2, $3, $4) RETURNING *",
    )
    .bind(&id)
    .bind(slug)
    .bind(&payload.title)
    .bind(&payload.description)
    .fetch_one(&pool)
    .await?;
    Ok((StatusCode::CREATED, Json(todo)))
}

#[utoipa::path(
    get,
    path = "/todos/{id}",
    responses(
        (status = 200, description = "Todo found", body = Todo),
        (status = 404, description = "Todo not found", body = ErrorResponse),
    ),
    tag = "todos"
)]
async fn get_todo(
    State(pool): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Todo>, AppError> {
    let todo = sqlx::query_as::<_, Todo>("SELECT * FROM todos WHERE id = $1")
        .bind(id)
        .fetch_optional(&pool)
        .await?
        .ok_or(AppError::NotFound)?; // Option → Result, None becomes AppError::NotFound
    Ok(Json(todo))
}

#[utoipa::path(
    put,
    path = "/todos/{id}",
    request_body = UpdateTodo,
    responses(
        (status = 200, description = "Todo updated", body = Todo),
        (status = 404, description = "Todo not found", body = ErrorResponse),
        (status = 409, description = "Slug already exists", body = ErrorResponse),
    ),
    tag = "todos"
)]
async fn update_todo(
    State(pool): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateTodo>,
) -> Result<Json<Todo>, AppError> {
    let mut slug = slugify(&payload.title);

    // check slog uniqueness before attempting insert and add ulid to the slug to guarantee uniqueness without relying on DB errors for control flow
    for attempt in 0..3 {
        if let Some(existing) = get_todo_by_slug(State(pool.clone()), slug.clone()).await? {
            if attempt == 2 {
                return Err(AppError::Conflict(format!(
                    "Failed to generate unique slug after 3 attempts"
                )));
            }
            // append a short random string to the slug and try again
            let random_suffix: String = Ulid::new().to_string()[20..].to_string();
            let new_slug = format!("{}-{}", slug, random_suffix);
            println!("Slug '{slug}' already exists, trying '{new_slug}'");
            slug = new_slug;
        } else {
            break; // slug is unique, proceed with insert
        }
    }

    let todo = sqlx::query_as::<_, Todo>(
        "UPDATE todos SET slug = $1, title = $2, description = $3, completed = $4 WHERE id = $5 RETURNING *",
    )
    .bind(slug)
    .bind(&payload.title)
    .bind(&payload.description)
    .bind(payload.completed)
    .bind(id)
    .fetch_optional(&pool)
    .await?
    .ok_or(AppError::NotFound)?;
    Ok(Json(todo))
}

async fn get_todo_by_slug(
    State(pool): State<AppState>,
    slug: String,
) -> Result<Option<Todo>, AppError> {
    let todo = sqlx::query_as::<_, Todo>("SELECT * FROM todos WHERE slug = $1")
        .bind(slug)
        .fetch_optional(&pool)
        .await?;
    Ok(todo)
}
