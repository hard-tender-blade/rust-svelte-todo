use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema, Clone, sqlx::FromRow)]
pub struct Todo {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub completed: bool,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateTodo {
    pub title: String,
    pub description: String,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateUser {
    pub full_name: String,
    pub email: String,
    pub password: String,
    pub role: UserRole,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateUser {
    pub full_name: String,
    pub role: UserRole,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateTodo {
    pub title: String,
    pub description: String,
    pub completed: bool,
}

#[derive(Serialize, ToSchema)]
pub struct Token {
    pub token: String,
}

#[derive(Deserialize, ToSchema)]
pub struct SignupUser {
    pub full_name: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize, ToSchema)]
pub struct SigninUser {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub id: String,
    pub exp: u64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    Editor,
    Viewer,
}

#[derive(Debug, Serialize, ToSchema, Clone, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub slug: String,
    pub full_name: String,
    pub email: String,
    pub role: UserRole,
}

#[derive(sqlx::FromRow)]
pub struct DBUser {
    pub id: String,
    pub slug: String,
    pub full_name: String,
    pub email: String,
    pub password: String,
    pub role: UserRole,
}

// ── Finance Plan ─────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, ToSchema, Clone, sqlx::FromRow)]
pub struct FinancePlanEntry {
    pub id: String,
    pub period_date: NaiveDate,
    pub income: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateFinancePlanEntry {
    pub period_date: NaiveDate,
    pub income: i32,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateFinancePlanEntry {
    pub period_date: NaiveDate,
    pub income: i32,
}

// ── Billing ──────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "billing_condition", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum BillingCondition {
    LessThan,
    MoreThan,
}

#[derive(Serialize, Deserialize, ToSchema, Clone, sqlx::FromRow)]
pub struct BillingEntry {
    pub id: String,
    pub fans_count: i32,
    pub condition: BillingCondition,
    pub basic_plan_price: Option<i32>,
    pub standard_plan_price: Option<i32>,
    pub premium_plan_price: Option<i32>,
    pub individual_plan_price: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize, ToSchema)]
pub struct CreateBillingEntry {
    pub fans_count: i32,
    pub condition: BillingCondition,
    pub basic_plan_price: Option<i32>,
    pub standard_plan_price: Option<i32>,
    pub premium_plan_price: Option<i32>,
    pub individual_plan_price: bool,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateBillingEntry {
    pub fans_count: i32,
    pub condition: BillingCondition,
    pub basic_plan_price: Option<i32>,
    pub standard_plan_price: Option<i32>,
    pub premium_plan_price: Option<i32>,
    pub individual_plan_price: bool,
}

// ── Admin Invoicing ───────────────────────────────────────────────────────────

/// A single invoicing entry logged against a tenant.
#[derive(Serialize, Deserialize, ToSchema, Clone, sqlx::FromRow)]
pub struct InvoicingEntry {
    pub id: String,
    /// MongoDB ObjectId of the tenant this invoice belongs to.
    pub mongo_id: String,
    pub date: NaiveDate,
    /// Price in integer units (e.g. cents).
    pub price: i32,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize, ToSchema)]
pub struct UpsertInvoicingEntry {
    pub mongo_id: String,
    pub date: NaiveDate,
    pub price: i32,
    pub note: Option<String>,
}

// ── Admin Onboarding ─────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "currencies", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum Currency {
    Czk,
    Eur,
}

/// A single onboarding entry logged against a tenant.
#[derive(Serialize, Deserialize, ToSchema, Clone, sqlx::FromRow)]
pub struct OnboardingEntry {
    pub id: String,
    /// MongoDB ObjectId of the tenant this onboarding belongs to.
    pub mongo_id: String,
    pub date_training: Option<NaiveDate>,
    pub paid: bool,
    pub price: i32,
    pub currency: Currency,
    pub invoiced: bool,
    pub invoiced_date: Option<NaiveDate>,
    pub business_module: bool,
    pub fans_module: bool,
    pub note: Option<String>,
    pub enigoo_involved: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize, ToSchema)]
pub struct UpsertOnboardingEntry {
    pub mongo_id: String,
    pub date_training: Option<NaiveDate>,
    pub paid: bool,
    pub price: i32,
    pub currency: Currency,
    pub invoiced: bool,
    pub invoiced_date: Option<NaiveDate>,
    pub business_module: bool,
    pub fans_module: bool,
    pub note: Option<String>,
    pub enigoo_involved: bool,
}

// ── Tenant Notes ──────────────────────────────────────────────────────────────

/// A free-text admin note attached to a specific tenant (identified by its MongoDB ObjectId).
#[derive(Serialize, Deserialize, ToSchema, Clone, sqlx::FromRow)]
pub struct TenantNote {
    pub id: String,
    /// MongoDB ObjectId of the tenant this note belongs to.
    pub mongo_id: String,
    pub note: Option<String>,
    /// Price note in integer units (e.g. cents).
    pub price: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize, ToSchema)]
pub struct UpsertTenantNote {
    pub note: Option<String>,
    /// Price note in integer units (e.g. cents).
    pub price: Option<i32>,
}
