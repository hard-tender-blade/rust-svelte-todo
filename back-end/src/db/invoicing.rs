use chrono::NaiveDate;

use crate::{error::AppError, models::InvoicingEntry};

use super::DatabaseService;

impl DatabaseService {
    /// List all invoicing entries across all tenants.
    pub async fn list_all_invoicing_entries(&self) -> Result<Vec<InvoicingEntry>, AppError> {
        let entries = sqlx::query_as!(
            InvoicingEntry,
            r#"SELECT id, mongo_id, date, price, note, created_at, updated_at
               FROM admin_invoicing_entries
               ORDER BY date DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(entries)
    }

    /// List all invoicing entries for a given tenant.
    pub async fn list_invoicing_entries(&self, mongo_id: &str) -> Result<Vec<InvoicingEntry>, AppError> {
        let entries = sqlx::query_as!(
            InvoicingEntry,
            r#"SELECT id, mongo_id, date, price, note, created_at, updated_at
               FROM admin_invoicing_entries
               WHERE mongo_id = $1
               ORDER BY date DESC"#,
            mongo_id,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(entries)
    }

    /// List all invoicing entries for a given tenant within a calendar month.
    ///
    /// `year` and `month` (1–12) define the month window.
    pub async fn list_invoicing_entries_for_month(
        &self,
        mongo_id: &str,
        year: i32,
        month: u32,
    ) -> Result<Vec<InvoicingEntry>, AppError> {
        // Build first and last day of the month. Using DATE_TRUNC on the DB side
        // is simpler than constructing boundaries in Rust.
        let entries = sqlx::query_as!(
            InvoicingEntry,
            r#"SELECT id, mongo_id, date, price, note, created_at, updated_at
               FROM admin_invoicing_entries
               WHERE mongo_id = $1
                 AND date >= DATE_TRUNC('month', MAKE_DATE($2, $3, 1))::DATE
                 AND date <  DATE_TRUNC('month', MAKE_DATE($2, $3, 1))::DATE + INTERVAL '1 month'
               ORDER BY date ASC"#,
            mongo_id,
            year,
            month as i32,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(entries)
    }

    /// Fetch a single invoicing entry by id.
    pub async fn get_invoicing_entry(&self, id: &str) -> Result<Option<InvoicingEntry>, AppError> {
        let entry = sqlx::query_as!(
            InvoicingEntry,
            r#"SELECT id, mongo_id, date, price, note, created_at, updated_at
               FROM admin_invoicing_entries
               WHERE id = $1"#,
            id,
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(entry)
    }

    /// Insert or update an invoicing entry by id.
    ///
    /// Uses `INSERT … ON CONFLICT (id) DO UPDATE` so callers can use the same
    /// endpoint for both create and update.
    pub async fn upsert_invoicing_entry(
        &self,
        id: &str,
        mongo_id: &str,
        date: NaiveDate,
        price: i32,
        note: Option<&str>,
    ) -> Result<InvoicingEntry, AppError> {
        let entry = sqlx::query_as!(
            InvoicingEntry,
            r#"INSERT INTO admin_invoicing_entries (id, mongo_id, date, price, note)
               VALUES ($1, $2, $3, $4, $5)
               ON CONFLICT (id) DO UPDATE
                   SET mongo_id   = EXCLUDED.mongo_id,
                       date       = EXCLUDED.date,
                       price      = EXCLUDED.price,
                       note       = EXCLUDED.note,
                       updated_at = NOW()
               RETURNING id, mongo_id, date, price, note, created_at, updated_at"#,
            id,
            mongo_id,
            date,
            price,
            note,
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(entry)
    }

    /// Delete an invoicing entry by id. Returns `true` if a row was removed.
    pub async fn delete_invoicing_entry(&self, id: &str) -> Result<bool, AppError> {
        let result = sqlx::query!(
            "DELETE FROM admin_invoicing_entries WHERE id = $1",
            id,
        )
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }
}
