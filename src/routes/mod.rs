pub mod todos;
pub mod users;

use axum::Router;
use sqlx::PgPool;
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_scalar::{Scalar, Servable};
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(tags(
    (name = "todos", description = "Todo management"),
    (name = "users", description = "User management"),
))]
pub struct ApiDoc;

pub fn build_router(pool: PgPool) -> Router {
    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(users::signup))
        .routes(routes!(users::signin))
        .routes(routes!(users::me))
        .routes(routes!(todos::list_todos, todos::create_todo))
        .routes(routes!(todos::get_todo, todos::update_todo))
        .with_state(pool)
        .split_for_parts();

    router
        .merge(SwaggerUi::new("/swagger").url("/openapi.json", api.clone()))
        .merge(Scalar::with_url("/scalar", api))
}
