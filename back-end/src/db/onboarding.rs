use chrono::NaiveDate;

use crate::{error::AppError, models::{Currency, OnboardingEntry}};

use super::DatabaseService;

impl DatabaseService {
    /// List all onboarding entries across all tenants.
    pub async fn list_all_onboarding_entries(&self) -> Result<Vec<OnboardingEntry>, AppError> {
        let entries = sqlx::query_as!(
            OnboardingEntry,
            r#"SELECT id, mongo_id, date_training, paid, price,
                      currency AS "currency: Currency",
                      invoiced, invoiced_date, business_module, fans_module,
                      note, enigoo_involved, created_at, updated_at
               FROM admin_onboarding_entries
               ORDER BY created_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(entries)
    }

    /// List all onboarding entries for a given tenant.
    pub async fn list_onboarding_entries(&self, mongo_id: &str) -> Result<Vec<OnboardingEntry>, AppError> {
        let entries = sqlx::query_as!(
            OnboardingEntry,
            r#"SELECT id, mongo_id, date_training, paid, price,
                      currency AS "currency: Currency",
                      invoiced, invoiced_date, business_module, fans_module,
                      note, enigoo_involved, created_at, updated_at
               FROM admin_onboarding_entries
               WHERE mongo_id = $1
               ORDER BY created_at DESC"#,
            mongo_id,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(entries)
    }

    /// Fetch a single onboarding entry by id.
    pub async fn get_onboarding_entry(&self, id: &str) -> Result<Option<OnboardingEntry>, AppError> {
        let entry = sqlx::query_as!(
            OnboardingEntry,
            r#"SELECT id, mongo_id, date_training, paid, price,
                      currency AS "currency: Currency",
                      invoiced, invoiced_date, business_module, fans_module,
                      note, enigoo_involved, created_at, updated_at
               FROM admin_onboarding_entries
               WHERE id = $1"#,
            id,
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(entry)
    }

    /// Insert or update an onboarding entry by id.
    pub async fn upsert_onboarding_entry(
        &self,
        id: &str,
        mongo_id: &str,
        date_training: Option<NaiveDate>,
        paid: bool,
        price: i32,
        currency: &Currency,
        invoiced: bool,
        invoiced_date: Option<NaiveDate>,
        business_module: bool,
        fans_module: bool,
        note: Option<&str>,
        enigoo_involved: bool,
    ) -> Result<OnboardingEntry, AppError> {
        let entry = sqlx::query_as!(
            OnboardingEntry,
            r#"INSERT INTO admin_onboarding_entries
                   (id, mongo_id, date_training, paid, price, currency,
                    invoiced, invoiced_date, business_module, fans_module,
                    note, enigoo_involved)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
               ON CONFLICT (id) DO UPDATE
                   SET mongo_id         = EXCLUDED.mongo_id,
                       date_training    = EXCLUDED.date_training,
                       paid             = EXCLUDED.paid,
                       price            = EXCLUDED.price,
                       currency         = EXCLUDED.currency,
                       invoiced         = EXCLUDED.invoiced,
                       invoiced_date    = EXCLUDED.invoiced_date,
                       business_module  = EXCLUDED.business_module,
                       fans_module      = EXCLUDED.fans_module,
                       note             = EXCLUDED.note,
                       enigoo_involved  = EXCLUDED.enigoo_involved,
                       updated_at       = NOW()
               RETURNING id, mongo_id, date_training, paid, price,
                         currency AS "currency: Currency",
                         invoiced, invoiced_date, business_module, fans_module,
                         note, enigoo_involved, created_at, updated_at"#,
            id,
            mongo_id,
            date_training,
            paid,
            price,
            currency as &Currency,
            invoiced,
            invoiced_date,
            business_module,
            fans_module,
            note,
            enigoo_involved,
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(entry)
    }

    /// Delete an onboarding entry by id. Returns `true` if a row was removed.
    pub async fn delete_onboarding_entry(&self, id: &str) -> Result<bool, AppError> {
        let result = sqlx::query!(
            "DELETE FROM admin_onboarding_entries WHERE id = $1",
            id,
        )
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }
}
