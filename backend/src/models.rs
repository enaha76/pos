use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use sqlx::FromRow;

// Field names are snake_case to match src/types/domain.ts on the client.

#[derive(Serialize, FromRow)]
pub struct Settings {
    pub spot_label: String,
    pub currency_symbol: String,
}

#[derive(Serialize, FromRow)]
pub struct Zone {
    pub zone_id: String,
    pub name: String,
    pub display_order: i32,
    pub table_mode: String,
    pub spot_label: Option<String>,
    pub active: bool,
}

#[derive(Serialize, FromRow)]
pub struct TableSpot {
    pub table_id: String,
    pub zone_id: String,
    pub label: String,
    pub active: bool,
}

#[derive(Serialize, FromRow)]
pub struct Server {
    pub server_id: String,
    pub name: String,
    pub active: bool,
}

#[derive(Serialize, FromRow)]
pub struct Shift {
    pub shift_id: String,
    pub name: String,
    pub start_time: String,
    pub end_time: String,
}

#[derive(Serialize, FromRow)]
pub struct ShiftAssignment {
    pub assignment_id: String,
    pub server_id: String,
    pub zone_id: String,
    pub shift_id: String,
    pub date: NaiveDate,
}

#[derive(Serialize, FromRow)]
pub struct Category {
    pub category_id: String,
    pub name: String,
    pub color: String,
    pub display_order: i32,
}

#[derive(Serialize, FromRow)]
pub struct Product {
    pub product_id: String,
    pub name: String,
    pub category_id: String,
    pub price: i32,
    pub active: bool,
}

#[derive(Serialize, FromRow)]
pub struct Modifier {
    pub modifier_id: String,
    pub product_id: String,
    pub name: String,
    pub price_delta: i32,
    /// Options in the same group are single-select; null = independent add-on.
    pub mod_group: Option<String>,
}

#[derive(Serialize, FromRow)]
pub struct ReasonCode {
    pub reason_id: String,
    pub kind: String,
    pub label: String,
    pub active: bool,
}

#[derive(Serialize, FromRow)]
pub struct Check {
    pub check_id: String,
    pub ticket_number: i32,
    pub zone_id: String,
    pub server_id: String,
    pub table_id: Option<String>,
    pub table_label: Option<String>,
    pub status: String,
    pub reason_id: Option<String>,
    pub opened_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, FromRow)]
pub struct OrderItem {
    pub item_id: String,
    pub check_id: String,
    pub server_id: String,
    pub product_id: String,
    pub name: String,
    pub qty: i32,
    pub unit_price: i32,
    pub state: String,
    pub reason_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, FromRow)]
pub struct Payment {
    pub payment_id: String,
    pub check_id: String,
    pub method: String,
    pub amount: i32,
    pub paid_at: DateTime<Utc>,
}

/// A check with its lines and payments nested (what the cashier UI consumes).
#[derive(Serialize)]
pub struct CheckWithLines {
    #[serde(flatten)]
    pub check: Check,
    pub items: Vec<OrderItem>,
    pub payments: Vec<Payment>,
}
