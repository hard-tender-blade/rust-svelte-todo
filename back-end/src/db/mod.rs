pub mod billing;
pub mod finance_plan;
pub mod invoicing;
pub mod onboarding;
pub mod tenant_notes;
pub mod todos;
pub mod users;

use sqlx::PgPool;

#[derive(Clone)]
pub struct DatabaseService {
    pool: PgPool,
}

impl DatabaseService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
