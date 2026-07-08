use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::response::Response;
use axum::Json;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{FromRow, PgPool};

use crate::error::AppError;
use crate::models::*;

pub async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

// ---- auth ----

#[derive(Deserialize)]
pub struct LoginReq {
    pub pin: String,
}

#[derive(Serialize)]
pub struct LoginResp {
    pub user_id: String,
    pub name: String,
    pub role: String,
}

#[derive(FromRow)]
struct UserAuth {
    user_id: String,
    name: String,
    pin_hash: String,
    role: String,
}

/// Validates a PIN against a user account's bcrypt hash; returns identity + role.
pub async fn login(
    State(pool): State<PgPool>,
    Json(b): Json<LoginReq>,
) -> Result<Json<LoginResp>, AppError> {
    let users = sqlx::query_as::<_, UserAuth>(
        "select user_id, name, pin_hash, role from users where active = true",
    )
    .fetch_all(&pool)
    .await?;

    for u in users {
        if bcrypt::verify(&b.pin, &u.pin_hash).unwrap_or(false) {
            return Ok(Json(LoginResp { user_id: u.user_id, name: u.name, role: u.role }));
        }
    }
    Err(AppError::Unauthorized)
}

// ---- realtime ----

/// Clients connect here; the server pushes a short message ("checks" / "config")
/// whenever data changes, and the client refetches.
pub async fn ws_handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    let mut rx = crate::events().subscribe();
    loop {
        tokio::select! {
            event = rx.recv() => match event {
                Ok(kind) => {
                    if socket.send(Message::Text(kind.into())).await.is_err() {
                        break; // client gone
                    }
                }
                Err(_) => {} // lagged — ignore, next event will resync
            },
            incoming = socket.recv() => match incoming {
                Some(Ok(_)) => {}    // ignore anything the client sends
                _ => break,          // closed / error
            },
        }
    }
}

/// One bootstrap call returning all owner-managed config (mirrors the client seed).
pub async fn get_config(State(pool): State<PgPool>) -> Result<Json<Value>, AppError> {
    let settings =
        sqlx::query_as::<_, Settings>("select spot_label, currency_symbol from settings where id = 1")
            .fetch_one(&pool)
            .await?;
    let zones = sqlx::query_as::<_, Zone>("select * from zones order by display_order")
        .fetch_all(&pool)
        .await?;
    let table_spots = sqlx::query_as::<_, TableSpot>("select * from table_spots")
        .fetch_all(&pool)
        .await?;
    let servers = sqlx::query_as::<_, Server>("select * from servers")
        .fetch_all(&pool)
        .await?;
    let shifts = sqlx::query_as::<_, Shift>("select * from shifts")
        .fetch_all(&pool)
        .await?;
    let categories =
        sqlx::query_as::<_, Category>("select * from categories order by display_order")
            .fetch_all(&pool)
            .await?;
    let products = sqlx::query_as::<_, Product>("select * from products")
        .fetch_all(&pool)
        .await?;
    let modifiers = sqlx::query_as::<_, Modifier>("select * from modifiers")
        .fetch_all(&pool)
        .await?;
    let reason_codes = sqlx::query_as::<_, ReasonCode>("select * from reason_codes")
        .fetch_all(&pool)
        .await?;
    let shift_assignments =
        sqlx::query_as::<_, ShiftAssignment>("select * from shift_assignments")
            .fetch_all(&pool)
            .await?;

    Ok(Json(json!({
        "settings": settings,
        "zones": zones,
        "tableSpots": table_spots,
        "servers": servers,
        "shifts": shifts,
        "categories": categories,
        "products": products,
        "modifiers": modifiers,
        "reasonCodes": reason_codes,
        "shiftAssignments": shift_assignments,
    })))
}

#[derive(Deserialize)]
pub struct CreateCheck {
    pub zone_id: String,
    pub server_id: String,
    pub table_id: Option<String>,
    pub table_label: Option<String>,
}

pub async fn create_check(
    State(pool): State<PgPool>,
    Json(b): Json<CreateCheck>,
) -> Result<Json<Check>, AppError> {
    let check = sqlx::query_as::<_, Check>(
        "insert into checks (zone_id, server_id, table_id, table_label) \
         values ($1, $2, $3, $4) returning *",
    )
    .bind(b.zone_id)
    .bind(b.server_id)
    .bind(b.table_id)
    .bind(b.table_label)
    .fetch_one(&pool)
    .await?;
    Ok(Json(check))
}

#[derive(Deserialize)]
pub struct AddItem {
    pub check_id: String,
    pub product_id: String,
    pub qty: Option<i32>,
    pub modifier_ids: Option<Vec<String>>,
}

pub async fn add_item(
    State(pool): State<PgPool>,
    Json(b): Json<AddItem>,
) -> Result<Json<OrderItem>, AppError> {
    let product =
        sqlx::query_as::<_, Product>("select * from products where product_id = $1 and active = true")
            .bind(&b.product_id)
            .fetch_optional(&pool)
            .await?
            .ok_or_else(|| AppError::BadRequest("unknown product".into()))?;

    let check = sqlx::query_as::<_, Check>("select * from checks where check_id = $1")
        .bind(&b.check_id)
        .fetch_optional(&pool)
        .await?
        .ok_or(AppError::NotFound)?;

    let qty = b.qty.unwrap_or(1).max(1);

    // fold any chosen modifiers into the line's snapshot name + price
    let mut line_name = product.name.clone();
    let mut line_price = product.price;
    if let Some(ids) = &b.modifier_ids {
        if !ids.is_empty() {
            let mods = sqlx::query_as::<_, (String, i32)>(
                "select name, price_delta from modifiers \
                 where product_id = $1 and modifier_id = any($2) order by name",
            )
            .bind(&product.product_id)
            .bind(&ids[..])
            .fetch_all(&pool)
            .await?;
            if !mods.is_empty() {
                let names: Vec<&str> = mods.iter().map(|(n, _)| n.as_str()).collect();
                line_name = format!("{} · {}", product.name, names.join(", "));
                line_price = product.price + mods.iter().map(|(_, d)| *d).sum::<i32>();
            }
        }
    }

    // tap-to-increment: merge into an existing HELD line with the same snapshot name
    let merged = sqlx::query_as::<_, OrderItem>(
        "update order_items set qty = qty + $3 \
         where item_id = (select item_id from order_items \
                          where check_id = $1 and name = $2 and state = 'HELD' \
                          order by created_at limit 1) returning *",
    )
    .bind(&check.check_id)
    .bind(&line_name)
    .bind(qty)
    .fetch_optional(&pool)
    .await?;

    let item = match merged {
        Some(it) => it,
        None => {
            // name + price snapshotted so menu changes never alter past bills
            sqlx::query_as::<_, OrderItem>(
                "insert into order_items (check_id, server_id, product_id, name, qty, unit_price, state) \
                 values ($1, $2, $3, $4, $5, $6, 'HELD') returning *",
            )
            .bind(&check.check_id)
            .bind(&check.server_id)
            .bind(&product.product_id)
            .bind(&line_name)
            .bind(qty)
            .bind(line_price)
            .fetch_one(&pool)
            .await?
        }
    };
    Ok(Json(item))
}

#[derive(Deserialize)]
pub struct CheckRef {
    pub check_id: String,
}

pub async fn send(
    State(pool): State<PgPool>,
    Json(b): Json<CheckRef>,
) -> Result<Json<CheckWithLines>, AppError> {
    let check = sqlx::query_as::<_, Check>("select * from checks where check_id = $1")
        .bind(&b.check_id)
        .fetch_optional(&pool)
        .await?
        .ok_or(AppError::NotFound)?;

    let res = sqlx::query("update order_items set state = 'SENT' where check_id = $1 and state = 'HELD'")
        .bind(&b.check_id)
        .execute(&pool)
        .await?;

    if res.rows_affected() > 0 {
        sqlx::query("update checks set status = 'IN_PROGRESS' where check_id = $1")
            .bind(&b.check_id)
            .execute(&pool)
            .await?;
        log_audit(
            &pool,
            Some(&check.server_id),
            "send",
            &b.check_id,
            None,
            Some(&format!("{} line(s)", res.rows_affected())),
        )
        .await?;
    }
    load_check(&pool, &b.check_id).await
}

#[derive(Deserialize)]
pub struct ItemReason {
    pub item_id: String,
    pub reason_id: String,
}

pub async fn void_item(
    State(pool): State<PgPool>,
    Json(b): Json<ItemReason>,
) -> Result<Json<OrderItem>, AppError> {
    let item = sqlx::query_as::<_, OrderItem>(
        "update order_items set state = 'VOID', reason_id = $2 where item_id = $1 returning *",
    )
    .bind(&b.item_id)
    .bind(&b.reason_id)
    .fetch_optional(&pool)
    .await?
    .ok_or(AppError::NotFound)?;
    log_audit(&pool, Some(&item.server_id), "void", &item.item_id, Some(&b.reason_id), None).await?;
    Ok(Json(item))
}

pub async fn comp_item(
    State(pool): State<PgPool>,
    Json(b): Json<ItemReason>,
) -> Result<Json<OrderItem>, AppError> {
    let item = sqlx::query_as::<_, OrderItem>(
        "update order_items set state = 'COMP', reason_id = $2 where item_id = $1 returning *",
    )
    .bind(&b.item_id)
    .bind(&b.reason_id)
    .fetch_optional(&pool)
    .await?
    .ok_or(AppError::NotFound)?;
    log_audit(&pool, Some(&item.server_id), "comp", &item.item_id, Some(&b.reason_id), None).await?;
    Ok(Json(item))
}

#[derive(Deserialize)]
pub struct Pay {
    pub check_id: String,
    pub method: String,
}

pub async fn pay(
    State(pool): State<PgPool>,
    Json(b): Json<Pay>,
) -> Result<Json<CheckWithLines>, AppError> {
    let check = sqlx::query_as::<_, Check>("select * from checks where check_id = $1")
        .bind(&b.check_id)
        .fetch_optional(&pool)
        .await?
        .ok_or(AppError::NotFound)?;

    let due: i32 = sqlx::query_scalar(
        "select coalesce(sum(qty * unit_price), 0)::int from order_items \
         where check_id = $1 and state in ('HELD','SENT')",
    )
    .bind(&b.check_id)
    .fetch_one(&pool)
    .await?;

    if due <= 0 {
        return Err(AppError::BadRequest("nothing to pay".into()));
    }

    sqlx::query("insert into payments (check_id, method, amount) values ($1, $2, $3)")
        .bind(&b.check_id)
        .bind(&b.method)
        .bind(due)
        .execute(&pool)
        .await?;
    sqlx::query("update checks set status = 'CLOSED_PAID', closed_at = now() where check_id = $1")
        .bind(&b.check_id)
        .execute(&pool)
        .await?;
    log_audit(
        &pool,
        Some(&check.server_id),
        "pay",
        &b.check_id,
        None,
        Some(&format!("{} {}", b.method, due)),
    )
    .await?;
    load_check(&pool, &b.check_id).await
}

#[derive(Deserialize)]
pub struct CloseUnpaid {
    pub check_id: String,
    pub reason_id: String,
}

pub async fn close_unpaid(
    State(pool): State<PgPool>,
    Json(b): Json<CloseUnpaid>,
) -> Result<Json<CheckWithLines>, AppError> {
    let check = sqlx::query_as::<_, Check>("select * from checks where check_id = $1")
        .bind(&b.check_id)
        .fetch_optional(&pool)
        .await?
        .ok_or(AppError::NotFound)?;
    sqlx::query(
        "update checks set status = 'CLOSED_UNPAID', reason_id = $2, closed_at = now() where check_id = $1",
    )
    .bind(&b.check_id)
    .bind(&b.reason_id)
    .execute(&pool)
    .await?;
    log_audit(&pool, Some(&check.server_id), "unpaid-close", &b.check_id, Some(&b.reason_id), None)
        .await?;
    load_check(&pool, &b.check_id).await
}

pub async fn list_open(State(pool): State<PgPool>) -> Result<Json<Vec<CheckWithLines>>, AppError> {
    let checks = sqlx::query_as::<_, Check>(
        "select * from checks where status in ('OPEN','IN_PROGRESS') order by ticket_number",
    )
    .fetch_all(&pool)
    .await?;

    let mut out = Vec::with_capacity(checks.len());
    for check in checks {
        let (items, payments) = lines_of(&pool, &check.check_id).await?;
        out.push(CheckWithLines { check, items, payments });
    }
    Ok(Json(out))
}

#[derive(Serialize, sqlx::FromRow)]
struct LabelAmount {
    label: String,
    amount: i64,
}

#[derive(Serialize, sqlx::FromRow)]
struct VoidCompRow {
    state: String,
    label: Option<String>,
    count: i64,
    amount: i64,
}

pub async fn reports_summary(State(pool): State<PgPool>) -> Result<Json<Value>, AppError> {
    let total_sales: i64 =
        sqlx::query_scalar("select coalesce(sum(amount), 0)::bigint from payments")
            .fetch_one(&pool)
            .await?;
    let paid_count: i64 =
        sqlx::query_scalar("select count(*) from checks where status = 'CLOSED_PAID'")
            .fetch_one(&pool)
            .await?;
    let unpaid_count: i64 =
        sqlx::query_scalar("select count(*) from checks where status = 'CLOSED_UNPAID'")
            .fetch_one(&pool)
            .await?;

    let by_zone = sqlx::query_as::<_, LabelAmount>(
        "select z.name as label, coalesce(sum(p.amount), 0)::bigint as amount \
         from checks c join payments p on p.check_id = c.check_id \
         join zones z on z.zone_id = c.zone_id \
         where c.status = 'CLOSED_PAID' group by z.name order by amount desc",
    )
    .fetch_all(&pool)
    .await?;

    let by_server = sqlx::query_as::<_, LabelAmount>(
        "select s.name as label, coalesce(sum(p.amount), 0)::bigint as amount \
         from checks c join payments p on p.check_id = c.check_id \
         join servers s on s.server_id = c.server_id \
         where c.status = 'CLOSED_PAID' group by s.name order by amount desc",
    )
    .fetch_all(&pool)
    .await?;

    let void_comp = sqlx::query_as::<_, VoidCompRow>(
        "select oi.state as state, rc.label as label, \
         sum(oi.qty)::bigint as count, sum(oi.qty * oi.unit_price)::bigint as amount \
         from order_items oi left join reason_codes rc on rc.reason_id = oi.reason_id \
         where oi.state in ('VOID','COMP') group by oi.state, rc.label",
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(json!({
        "totalSales": total_sales,
        "paidChecks": paid_count,
        "unpaidChecks": unpaid_count,
        "salesByZone": by_zone,
        "salesByServer": by_server,
        "voidComp": void_comp,
    })))
}

// ---- item quantity ----

#[derive(Deserialize)]
pub struct SetQty {
    pub item_id: String,
    pub qty: i32,
}

pub async fn set_item_qty(
    State(pool): State<PgPool>,
    Json(b): Json<SetQty>,
) -> Result<Json<Value>, AppError> {
    // only HELD lines are editable; qty <= 0 removes the line
    if b.qty <= 0 {
        sqlx::query("delete from order_items where item_id = $1 and state = 'HELD'")
            .bind(&b.item_id)
            .execute(&pool)
            .await?;
        return Ok(Json(json!({ "deleted": true })));
    }
    let item = sqlx::query_as::<_, OrderItem>(
        "update order_items set qty = $2 where item_id = $1 and state = 'HELD' returning *",
    )
    .bind(&b.item_id)
    .bind(b.qty)
    .fetch_optional(&pool)
    .await?
    .ok_or(AppError::NotFound)?;
    Ok(Json(json!(item)))
}

// ---- reassign server / table on an open check ----

#[derive(Deserialize)]
pub struct SetServer {
    pub check_id: String,
    pub server_id: String,
}

pub async fn set_check_server(
    State(pool): State<PgPool>,
    Json(b): Json<SetServer>,
) -> Result<Json<CheckWithLines>, AppError> {
    sqlx::query("update checks set server_id = $2 where check_id = $1")
        .bind(&b.check_id)
        .bind(&b.server_id)
        .execute(&pool)
        .await?;
    // a check belongs to one server — keep its lines in sync
    sqlx::query("update order_items set server_id = $2 where check_id = $1")
        .bind(&b.check_id)
        .bind(&b.server_id)
        .execute(&pool)
        .await?;
    load_check(&pool, &b.check_id).await
}

#[derive(Deserialize)]
pub struct SetTable {
    pub check_id: String,
    pub table_id: Option<String>,
    pub table_label: Option<String>,
}

pub async fn set_check_table(
    State(pool): State<PgPool>,
    Json(b): Json<SetTable>,
) -> Result<Json<CheckWithLines>, AppError> {
    sqlx::query("update checks set table_id = $2, table_label = $3 where check_id = $1")
        .bind(&b.check_id)
        .bind(&b.table_id)
        .bind(&b.table_label)
        .execute(&pool)
        .await?;
    load_check(&pool, &b.check_id).await
}

// ---- owner-managed config writes ----

#[derive(Deserialize)]
pub struct SettingsPatch {
    pub spot_label: Option<String>,
    pub currency_symbol: Option<String>,
}

pub async fn update_settings(
    State(pool): State<PgPool>,
    Json(b): Json<SettingsPatch>,
) -> Result<Json<Settings>, AppError> {
    let s = sqlx::query_as::<_, Settings>(
        "update settings set spot_label = coalesce($1, spot_label), \
         currency_symbol = coalesce($2, currency_symbol) where id = 1 \
         returning spot_label, currency_symbol",
    )
    .bind(b.spot_label)
    .bind(b.currency_symbol)
    .fetch_one(&pool)
    .await?;
    Ok(Json(s))
}

#[derive(Deserialize)]
pub struct ZoneIn {
    pub zone_id: String,
    pub name: String,
    pub display_order: i32,
    pub table_mode: String,
    pub spot_label: Option<String>,
    pub active: bool,
}

pub async fn upsert_zone(
    State(pool): State<PgPool>,
    Json(b): Json<ZoneIn>,
) -> Result<Json<Zone>, AppError> {
    let z = sqlx::query_as::<_, Zone>(
        "insert into zones (zone_id, name, display_order, table_mode, spot_label, active) \
         values ($1, $2, $3, $4, $5, $6) \
         on conflict (zone_id) do update set name = excluded.name, \
           display_order = excluded.display_order, table_mode = excluded.table_mode, \
           spot_label = excluded.spot_label, active = excluded.active \
         returning *",
    )
    .bind(&b.zone_id)
    .bind(&b.name)
    .bind(b.display_order)
    .bind(&b.table_mode)
    .bind(&b.spot_label)
    .bind(b.active)
    .fetch_one(&pool)
    .await?;
    Ok(Json(z))
}

#[derive(Deserialize)]
pub struct CategoryIn {
    pub category_id: String,
    pub name: String,
    pub color: String,
    pub display_order: i32,
}

pub async fn upsert_category(
    State(pool): State<PgPool>,
    Json(b): Json<CategoryIn>,
) -> Result<Json<Category>, AppError> {
    let c = sqlx::query_as::<_, Category>(
        "insert into categories (category_id, name, color, display_order) \
         values ($1, $2, $3, $4) \
         on conflict (category_id) do update set name = excluded.name, \
           color = excluded.color, display_order = excluded.display_order \
         returning *",
    )
    .bind(&b.category_id)
    .bind(&b.name)
    .bind(&b.color)
    .bind(b.display_order)
    .fetch_one(&pool)
    .await?;
    Ok(Json(c))
}

#[derive(Deserialize)]
pub struct ProductIn {
    pub product_id: String,
    pub name: String,
    pub category_id: String,
    pub price: i32,
    pub active: bool,
}

pub async fn upsert_product(
    State(pool): State<PgPool>,
    Json(b): Json<ProductIn>,
) -> Result<Json<Product>, AppError> {
    let p = sqlx::query_as::<_, Product>(
        "insert into products (product_id, name, category_id, price, active) \
         values ($1, $2, $3, $4, $5) \
         on conflict (product_id) do update set name = excluded.name, \
           category_id = excluded.category_id, price = excluded.price, active = excluded.active \
         returning *",
    )
    .bind(&b.product_id)
    .bind(&b.name)
    .bind(&b.category_id)
    .bind(b.price)
    .bind(b.active)
    .fetch_one(&pool)
    .await?;
    Ok(Json(p))
}

#[derive(Deserialize)]
pub struct ServerIn {
    pub server_id: String,
    pub name: String,
    pub active: bool,
}

pub async fn upsert_server(
    State(pool): State<PgPool>,
    Json(b): Json<ServerIn>,
) -> Result<Json<Server>, AppError> {
    let s = sqlx::query_as::<_, Server>(
        "insert into servers (server_id, name, active) \
         values ($1, $2, $3) \
         on conflict (server_id) do update set name = excluded.name, active = excluded.active \
         returning *",
    )
    .bind(&b.server_id)
    .bind(&b.name)
    .bind(b.active)
    .fetch_one(&pool)
    .await?;
    Ok(Json(s))
}

#[derive(Deserialize)]
pub struct ShiftIn {
    pub shift_id: String,
    pub name: String,
    pub start_time: String,
    pub end_time: String,
}

pub async fn upsert_shift(
    State(pool): State<PgPool>,
    Json(b): Json<ShiftIn>,
) -> Result<Json<Shift>, AppError> {
    let s = sqlx::query_as::<_, Shift>(
        "insert into shifts (shift_id, name, start_time, end_time) \
         values ($1, $2, $3, $4) \
         on conflict (shift_id) do update set name = excluded.name, \
           start_time = excluded.start_time, end_time = excluded.end_time \
         returning *",
    )
    .bind(&b.shift_id)
    .bind(&b.name)
    .bind(&b.start_time)
    .bind(&b.end_time)
    .fetch_one(&pool)
    .await?;
    Ok(Json(s))
}

#[derive(Deserialize)]
pub struct ToggleAssignment {
    pub server_id: String,
    pub zone_id: String,
    pub shift_id: String,
    pub date: String,
}

pub async fn toggle_assignment(
    State(pool): State<PgPool>,
    Json(b): Json<ToggleAssignment>,
) -> Result<Json<Value>, AppError> {
    let date = NaiveDate::parse_from_str(&b.date, "%Y-%m-%d")
        .map_err(|_| AppError::BadRequest("date must be YYYY-MM-DD".into()))?;
    let existing: Option<String> = sqlx::query_scalar(
        "select assignment_id from shift_assignments \
         where server_id = $1 and zone_id = $2 and shift_id = $3 and date = $4",
    )
    .bind(&b.server_id)
    .bind(&b.zone_id)
    .bind(&b.shift_id)
    .bind(date)
    .fetch_optional(&pool)
    .await?;

    match existing {
        Some(id) => {
            sqlx::query("delete from shift_assignments where assignment_id = $1")
                .bind(id)
                .execute(&pool)
                .await?;
            Ok(Json(json!({ "assigned": false })))
        }
        None => {
            sqlx::query(
                "insert into shift_assignments (server_id, zone_id, shift_id, date) \
                 values ($1, $2, $3, $4)",
            )
            .bind(&b.server_id)
            .bind(&b.zone_id)
            .bind(&b.shift_id)
            .bind(date)
            .execute(&pool)
            .await?;
            Ok(Json(json!({ "assigned": true })))
        }
    }
}

// ---- daily report (admin only) ----

#[derive(Deserialize)]
pub struct DailyQuery {
    pub date: Option<String>,
}

#[derive(Serialize, FromRow)]
struct UnpaidRow {
    ticket_number: i32,
    server: String,
    zone: String,
    reason: String,
    amount: i64,
}

/// One day's activity, with the unpaid checks and who is responsible for each.
pub async fn reports_daily(
    State(pool): State<PgPool>,
    Query(q): Query<DailyQuery>,
) -> Result<Json<Value>, AppError> {
    // Mauritania is UTC, so opened_at::date (UTC) equals the local day.
    let date: chrono::NaiveDate = match q.date {
        Some(d) => chrono::NaiveDate::parse_from_str(&d, "%Y-%m-%d")
            .map_err(|_| AppError::BadRequest("date invalide".into()))?,
        None => sqlx::query_scalar("select current_date").fetch_one(&pool).await?,
    };

    let sales: i64 = sqlx::query_scalar(
        "select coalesce(sum(p.amount), 0)::bigint from payments p \
         join checks c on c.check_id = p.check_id \
         where c.status = 'CLOSED_PAID' and c.opened_at::date = $1",
    )
    .bind(date)
    .fetch_one(&pool)
    .await?;

    let paid_count: i64 = sqlx::query_scalar(
        "select count(*) from checks where status = 'CLOSED_PAID' and opened_at::date = $1",
    )
    .bind(date)
    .fetch_one(&pool)
    .await?;

    // The important part: unpaid checks + the responsible server.
    let unpaid = sqlx::query_as::<_, UnpaidRow>(
        "select c.ticket_number, s.name as server, z.name as zone, \
           coalesce(rc.label, '—') as reason, \
           coalesce((select sum(oi.qty * oi.unit_price) from order_items oi \
                     where oi.check_id = c.check_id and oi.state in ('HELD','SENT')), 0)::bigint as amount \
         from checks c \
         join servers s on s.server_id = c.server_id \
         join zones z on z.zone_id = c.zone_id \
         left join reason_codes rc on rc.reason_id = c.reason_id \
         where c.status = 'CLOSED_UNPAID' and c.opened_at::date = $1 \
         order by c.ticket_number",
    )
    .bind(date)
    .fetch_all(&pool)
    .await?;

    let by_server = sqlx::query_as::<_, LabelAmount>(
        "select s.name as label, coalesce(sum(p.amount), 0)::bigint as amount \
         from checks c join payments p on p.check_id = c.check_id \
         join servers s on s.server_id = c.server_id \
         where c.status = 'CLOSED_PAID' and c.opened_at::date = $1 \
         group by s.name order by amount desc",
    )
    .bind(date)
    .fetch_all(&pool)
    .await?;

    let void_comp = sqlx::query_as::<_, VoidCompRow>(
        "select oi.state as state, rc.label as label, \
           sum(oi.qty)::bigint as count, sum(oi.qty * oi.unit_price)::bigint as amount \
         from order_items oi join checks c on c.check_id = oi.check_id \
         left join reason_codes rc on rc.reason_id = oi.reason_id \
         where oi.state in ('VOID','COMP') and c.opened_at::date = $1 \
         group by oi.state, rc.label",
    )
    .bind(date)
    .fetch_all(&pool)
    .await?;

    Ok(Json(json!({
        "date": date.to_string(),
        "sales": sales,
        "paidCount": paid_count,
        "unpaidCount": unpaid.len(),
        "unpaid": unpaid,
        "byServer": by_server,
        "voidComp": void_comp,
    })))
}

// ---- user accounts (admin only) ----

#[derive(Serialize, FromRow)]
pub struct UserView {
    pub user_id: String,
    pub name: String,
    pub role: String,
    pub active: bool,
}

pub async fn list_users(State(pool): State<PgPool>) -> Result<Json<Vec<UserView>>, AppError> {
    let users =
        sqlx::query_as::<_, UserView>("select user_id, name, role, active from users order by name")
            .fetch_all(&pool)
            .await?;
    Ok(Json(users))
}

fn valid_role(role: &str) -> bool {
    role == "cashier" || role == "admin"
}

/// Would this PIN match an existing active user (other than `exclude`)?
/// bcrypt hashes are salted, so we must verify against each rather than compare.
async fn pin_collides(pool: &PgPool, pin: &str, exclude: Option<&str>) -> Result<bool, AppError> {
    let rows: Vec<(String, String)> =
        sqlx::query_as("select user_id, pin_hash from users where active = true")
            .fetch_all(pool)
            .await?;
    for (uid, hash) in rows {
        if exclude == Some(uid.as_str()) {
            continue;
        }
        if bcrypt::verify(pin, &hash).unwrap_or(false) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn hash_pin(pin: &str) -> Result<String, AppError> {
    if pin.len() < 4 {
        return Err(AppError::BadRequest("le code doit faire au moins 4 chiffres".into()));
    }
    bcrypt::hash(pin, bcrypt::DEFAULT_COST).map_err(|e| AppError::BadRequest(e.to_string()))
}

#[derive(Deserialize)]
pub struct CreateUser {
    pub name: String,
    pub role: String,
    pub pin: String,
}

pub async fn create_user(
    State(pool): State<PgPool>,
    Json(b): Json<CreateUser>,
) -> Result<Json<UserView>, AppError> {
    if b.name.trim().is_empty() {
        return Err(AppError::BadRequest("nom requis".into()));
    }
    if !valid_role(&b.role) {
        return Err(AppError::BadRequest("rôle invalide".into()));
    }
    if pin_collides(&pool, &b.pin, None).await? {
        return Err(AppError::BadRequest("ce code est déjà utilisé".into()));
    }
    let hash = hash_pin(&b.pin)?;
    let user = sqlx::query_as::<_, UserView>(
        "insert into users (name, role, pin_hash) values ($1, $2, $3) \
         returning user_id, name, role, active",
    )
    .bind(b.name.trim())
    .bind(&b.role)
    .bind(&hash)
    .fetch_one(&pool)
    .await?;
    Ok(Json(user))
}

#[derive(Deserialize)]
pub struct UpdateUser {
    pub user_id: String,
    pub name: String,
    pub role: String,
    pub active: bool,
}

pub async fn update_user(
    State(pool): State<PgPool>,
    Json(b): Json<UpdateUser>,
) -> Result<Json<UserView>, AppError> {
    if b.name.trim().is_empty() {
        return Err(AppError::BadRequest("nom requis".into()));
    }
    if !valid_role(&b.role) {
        return Err(AppError::BadRequest("rôle invalide".into()));
    }
    // never demote/deactivate the last active admin (lockout guard)
    if b.role != "admin" || !b.active {
        let other_admins: i64 = sqlx::query_scalar(
            "select count(*) from users where role = 'admin' and active = true and user_id <> $1",
        )
        .bind(&b.user_id)
        .fetch_one(&pool)
        .await?;
        if other_admins == 0 {
            return Err(AppError::BadRequest(
                "au moins un administrateur actif est requis".into(),
            ));
        }
    }
    let user = sqlx::query_as::<_, UserView>(
        "update users set name = $2, role = $3, active = $4 where user_id = $1 \
         returning user_id, name, role, active",
    )
    .bind(&b.user_id)
    .bind(b.name.trim())
    .bind(&b.role)
    .bind(b.active)
    .fetch_optional(&pool)
    .await?
    .ok_or(AppError::NotFound)?;
    Ok(Json(user))
}

#[derive(Deserialize)]
pub struct SetUserPin {
    pub user_id: String,
    pub pin: String,
}

pub async fn set_user_pin(
    State(pool): State<PgPool>,
    Json(b): Json<SetUserPin>,
) -> Result<Json<Value>, AppError> {
    if pin_collides(&pool, &b.pin, Some(&b.user_id)).await? {
        return Err(AppError::BadRequest("ce code est déjà utilisé".into()));
    }
    let hash = hash_pin(&b.pin)?;
    let res = sqlx::query("update users set pin_hash = $2 where user_id = $1")
        .bind(&b.user_id)
        .bind(&hash)
        .execute(&pool)
        .await?;
    if res.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(Json(json!({ "ok": true })))
}

// ---- helpers ----

async fn lines_of(
    pool: &PgPool,
    check_id: &str,
) -> Result<(Vec<OrderItem>, Vec<Payment>), AppError> {
    let items = sqlx::query_as::<_, OrderItem>(
        "select * from order_items where check_id = $1 order by created_at",
    )
    .bind(check_id)
    .fetch_all(pool)
    .await?;
    let payments = sqlx::query_as::<_, Payment>("select * from payments where check_id = $1")
        .bind(check_id)
        .fetch_all(pool)
        .await?;
    Ok((items, payments))
}

async fn load_check(pool: &PgPool, check_id: &str) -> Result<Json<CheckWithLines>, AppError> {
    let check = sqlx::query_as::<_, Check>("select * from checks where check_id = $1")
        .bind(check_id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound)?;
    let (items, payments) = lines_of(pool, check_id).await?;
    Ok(Json(CheckWithLines { check, items, payments }))
}

async fn log_audit(
    pool: &PgPool,
    actor_id: Option<&str>,
    action: &str,
    target: &str,
    reason_id: Option<&str>,
    detail: Option<&str>,
) -> Result<(), AppError> {
    sqlx::query(
        "insert into audit_log (actor_id, action, target, reason_id, detail) \
         values ($1, $2, $3, $4, $5)",
    )
    .bind(actor_id)
    .bind(action)
    .bind(target)
    .bind(reason_id)
    .bind(detail)
    .execute(pool)
    .await?;
    Ok(())
}
