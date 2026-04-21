mod auth;
mod db;
mod error;
mod models;
mod routes;

use sqlx::PgPool;

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

    let app: axum::Router = routes::build_router(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Listening on http://localhost:3000");
    println!("Swagger UI at http://localhost:3000/swagger/");
    println!("Scalar UI at http://localhost:3000/scalar/");
    println!("OpenAPI JSON at http://localhost:3000/openapi.json");
    axum::serve(listener, app).await.unwrap();
}
