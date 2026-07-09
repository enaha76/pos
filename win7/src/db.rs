//! Embedded SQLite layer for the native Windows 7 build.
//! Reuses the exact same schema, seed and menu as the Tauri build
//! (../../src-tauri/migrations/*.sql) and mirrors its check lifecycle.

use rusqlite::{params, params_from_iter, Connection, OptionalExtension};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

pub struct Db {
    conn: Connection,
}

pub struct Product {
    pub id: String,
    pub name: String,
    pub price: i64,
    pub category_id: String,
}

pub struct Category {
    pub id: String,
    pub name: String,
    pub color: String,
}

pub struct Zone {
    pub id: String,
    pub name: String,
    pub table_mode: String, // none | free | fixed
    pub spot_label: Option<String>,
}

/// Generic id + label row (used for table spots and servers).
pub struct NamedRow {
    pub id: String,
    pub label: String,
}

pub struct OpenCheck {
    pub check_id: String,
    pub ticket_number: i64,
    pub zone: String,
    pub table_label: String,
    pub server: String,
    pub total: i64,
}

pub struct UserRow {
    pub user_id: String,
    pub name: String,
    pub role: String,
}

pub struct User {
    pub id: String,
    pub name: String,
    pub role: String,
    pub active: bool,
}

pub struct AdminServer {
    pub id: String,
    pub name: String,
    pub active: bool,
}

pub struct Modifier {
    pub id: String,
    pub name: String,
    pub price_delta: i64,
    pub mod_group: Option<String>,
}

pub struct CheckItem {
    pub item_id: String,
    pub name: String,
    pub qty: i64,
    pub unit_price: i64,
    pub state: String, // HELD | SENT | VOID | COMP
}

pub struct CheckData {
    pub check_id: String,
    pub ticket_number: i64,
    pub status: String,
    pub zone_id: String,
    pub server_id: String,
    pub table_id: Option<String>,
    pub table_label: Option<String>,
    pub items: Vec<CheckItem>,
}

pub struct Bar {
    pub label: String,
    pub amount: i64,
}

pub struct VoidComp {
    pub state: String,
    pub label: String,
    pub count: i64,
    pub amount: i64,
}

pub struct Unpaid {
    pub ticket: i64,
    pub server: String,
    pub zone: String,
    pub reason: String,
    pub amount: i64,
}

pub struct DailyReport {
    pub date: String,
    pub sales: i64,
    pub paid_count: i64,
    pub unpaid_count: i64,
    pub unpaid: Vec<Unpaid>,
    pub by_server: Vec<Bar>,
    pub void_comp: Vec<VoidComp>,
}

pub struct GlobalReport {
    pub total_sales: i64,
    pub paid_checks: i64,
    pub unpaid_checks: i64,
    pub by_zone: Vec<Bar>,
    pub by_server: Vec<Bar>,
    pub void_comp: Vec<VoidComp>,
}

pub fn sha256_hex(s: &str) -> String {
    let mut h = Sha256::new();
    h.update(s.as_bytes());
    h.finalize().iter().map(|b| format!("{:02x}", b)).collect()
}

fn data_dir() -> PathBuf {
    let base = std::env::var("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let dir = base.join("CafeAdalyaCaisse");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

fn new_id(prefix: &str) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{}_{:x}{:x}", prefix, nanos, n)
}

const NOW: &str = "strftime('%Y-%m-%dT%H:%M:%fZ','now')";

impl Db {
    pub fn open() -> rusqlite::Result<Self> {
        let conn = Connection::open(data_dir().join("caisse.db"))?;
        // Enforce FKs (cascade deletes), and harden for a POS that may lose power.
        conn.execute_batch(
            "PRAGMA foreign_keys = ON; PRAGMA journal_mode = WAL; PRAGMA busy_timeout = 5000;",
        )?;
        let db = Db { conn };
        db.migrate()?;
        db.hash_seed_pins()?;
        Ok(db)
    }

    fn migrate(&self) -> rusqlite::Result<()> {
        let v: i64 = self.conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        if v < 1 {
            self.conn
                .execute_batch(include_str!("../../src-tauri/migrations/0001_init.sql"))?;
            self.conn
                .execute_batch(include_str!("../../src-tauri/migrations/0002_seed.sql"))?;
            self.conn
                .execute_batch(include_str!("../../src-tauri/migrations/0003_menu_adalya.sql"))?;
            self.conn.execute_batch("PRAGMA user_version = 1;")?;
        }
        Ok(())
    }

    fn hash_seed_pins(&self) -> rusqlite::Result<()> {
        let rows: Vec<(String, String)> = {
            let mut stmt = self.conn.prepare("select user_id, pin_hash from users")?;
            let it = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;
            it.collect::<rusqlite::Result<Vec<_>>>()?
        };
        for (uid, ph) in rows {
            let hashed = ph.len() == 64 && ph.bytes().all(|b| b.is_ascii_hexdigit());
            if !hashed {
                self.conn.execute(
                    "update users set pin_hash = ?1 where user_id = ?2",
                    params![sha256_hex(&ph), uid],
                )?;
            }
        }
        Ok(())
    }

    pub fn login(&self, pin: &str) -> rusqlite::Result<Option<UserRow>> {
        let h = sha256_hex(pin);
        let mut stmt = self
            .conn
            .prepare("select user_id, name, role from users where active = 1 and pin_hash = ?1 limit 1")?;
        let mut rows = stmt.query(params![h])?;
        match rows.next()? {
            Some(r) => Ok(Some(UserRow {
                user_id: r.get(0)?,
                name: r.get(1)?,
                role: r.get(2)?,
            })),
            None => Ok(None),
        }
    }

    pub fn modifiers(&self, product_id: &str) -> rusqlite::Result<Vec<Modifier>> {
        let mut stmt = self.conn.prepare(
            "select modifier_id, name, price_delta, mod_group from modifiers where product_id = ?1 order by mod_group, name",
        )?;
        let it = stmt.query_map(params![product_id], |r| {
            Ok(Modifier {
                id: r.get(0)?,
                name: r.get(1)?,
                price_delta: r.get(2)?,
                mod_group: r.get(3)?,
            })
        })?;
        it.collect()
    }

    pub fn settings(&self) -> rusqlite::Result<(String, String)> {
        self.conn.query_row(
            "select spot_label, currency_symbol from settings where id = 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
    }

    pub fn categories(&self) -> rusqlite::Result<Vec<Category>> {
        let mut stmt = self
            .conn
            .prepare("select category_id, name, color from categories order by display_order")?;
        let it = stmt.query_map([], |r| {
            Ok(Category {
                id: r.get(0)?,
                name: r.get(1)?,
                color: r.get(2)?,
            })
        })?;
        it.collect()
    }

    pub fn products(&self) -> rusqlite::Result<Vec<Product>> {
        let mut stmt = self.conn.prepare(
            "select p.product_id, p.name, p.price, p.category_id from products p \
             join categories c on c.category_id = p.category_id \
             where p.active = 1 order by c.display_order, p.name",
        )?;
        let it = stmt.query_map([], |r| {
            Ok(Product {
                id: r.get(0)?,
                name: r.get(1)?,
                price: r.get(2)?,
                category_id: r.get(3)?,
            })
        })?;
        it.collect()
    }

    pub fn zones(&self) -> rusqlite::Result<Vec<Zone>> {
        let mut stmt = self.conn.prepare(
            "select zone_id, name, table_mode, spot_label from zones where active = 1 order by display_order",
        )?;
        let it = stmt.query_map([], |r| {
            Ok(Zone {
                id: r.get(0)?,
                name: r.get(1)?,
                table_mode: r.get(2)?,
                spot_label: r.get(3)?,
            })
        })?;
        it.collect()
    }

    pub fn tables(&self, zone_id: &str) -> rusqlite::Result<Vec<NamedRow>> {
        let mut stmt = self
            .conn
            .prepare("select table_id, label from table_spots where zone_id = ?1 and active = 1")?;
        let it = stmt.query_map(params![zone_id], |r| {
            Ok(NamedRow {
                id: r.get(0)?,
                label: r.get(1)?,
            })
        })?;
        it.collect()
    }

    pub fn all_tables(&self) -> rusqlite::Result<Vec<NamedRow>> {
        let mut stmt = self.conn.prepare("select table_id, label from table_spots")?;
        let it = stmt.query_map([], |r| {
            Ok(NamedRow {
                id: r.get(0)?,
                label: r.get(1)?,
            })
        })?;
        it.collect()
    }

    pub fn servers(&self) -> rusqlite::Result<Vec<NamedRow>> {
        let mut stmt = self
            .conn
            .prepare("select server_id, name from servers where active = 1 order by name")?;
        let it = stmt.query_map([], |r| {
            Ok(NamedRow {
                id: r.get(0)?,
                label: r.get(1)?,
            })
        })?;
        it.collect()
    }

    pub fn shifts(&self) -> rusqlite::Result<Vec<(String, String, String)>> {
        let mut stmt = self.conn.prepare("select shift_id, start_time, end_time from shifts")?;
        let it = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?;
        it.collect()
    }

    /// Servers rostered to a zone for a shift, today (matches the seed's date('now')).
    pub fn rostered_servers(&self, zone_id: &str, shift_id: &str) -> rusqlite::Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "select server_id from shift_assignments where zone_id = ?1 and shift_id = ?2 and date = date('now')",
        )?;
        let it = stmt.query_map(params![zone_id, shift_id], |r| r.get(0))?;
        it.collect()
    }

    pub fn reason_codes(&self, kind: &str) -> rusqlite::Result<Vec<(String, String)>> {
        let mut stmt = self
            .conn
            .prepare("select reason_id, label from reason_codes where kind = ?1 and active = 1")?;
        let it = stmt.query_map(params![kind], |r| Ok((r.get(0)?, r.get(1)?)))?;
        it.collect()
    }

    pub fn open_checks(&self) -> rusqlite::Result<Vec<OpenCheck>> {
        let mut stmt = self.conn.prepare(
            "select c.check_id, c.ticket_number, z.name, \
                coalesce(c.table_label, coalesce(t.label, '')), s.name, \
                coalesce((select sum(oi.qty*oi.unit_price) from order_items oi where oi.check_id = c.check_id and oi.state in ('HELD','SENT')), 0) \
             from checks c \
             join zones z on z.zone_id = c.zone_id \
             join servers s on s.server_id = c.server_id \
             left join table_spots t on t.table_id = c.table_id \
             where c.status in ('OPEN','IN_PROGRESS') order by c.ticket_number",
        )?;
        let it = stmt.query_map([], |r| {
            Ok(OpenCheck {
                check_id: r.get(0)?,
                ticket_number: r.get(1)?,
                zone: r.get(2)?,
                table_label: r.get(3)?,
                server: r.get(4)?,
                total: r.get(5)?,
            })
        })?;
        it.collect()
    }

    // ---- check lifecycle (mirrors src/lib/apiSqlite.ts) ----

    pub fn create_check(
        &self,
        zone_id: &str,
        server_id: &str,
        table_id: Option<&str>,
        table_label: Option<&str>,
    ) -> rusqlite::Result<String> {
        let ticket: i64 =
            self.conn
                .query_row("select coalesce(max(ticket_number),0)+1 from checks", [], |r| r.get(0))?;
        let id = new_id("chk");
        self.conn.execute(
            &format!(
                "insert into checks (check_id, ticket_number, zone_id, server_id, table_id, table_label, status, opened_at) \
                 values (?1, ?2, ?3, ?4, ?5, ?6, 'OPEN', {NOW})"
            ),
            params![id, ticket, zone_id, server_id, table_id, table_label],
        )?;
        Ok(id)
    }

    pub fn set_check_table(
        &self,
        check_id: &str,
        table_id: Option<&str>,
        table_label: Option<&str>,
    ) -> rusqlite::Result<()> {
        self.conn.execute(
            "update checks set table_id = ?2, table_label = ?3 where check_id = ?1",
            params![check_id, table_id, table_label],
        )?;
        Ok(())
    }

    pub fn set_check_server(&self, check_id: &str, server_id: &str) -> rusqlite::Result<()> {
        self.conn
            .execute("update checks set server_id = ?2 where check_id = ?1", params![check_id, server_id])?;
        self.conn.execute(
            "update order_items set server_id = ?2 where check_id = ?1",
            params![check_id, server_id],
        )?;
        Ok(())
    }

    pub fn add_item(
        &self,
        check_id: &str,
        product_id: &str,
        qty: i64,
        modifier_ids: &[String],
    ) -> rusqlite::Result<()> {
        let (base_name, base_price): (String, i64) = self.conn.query_row(
            "select name, price from products where product_id = ?1 and active = 1",
            params![product_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        // Fold any chosen modifiers into the line name + unit price (sorted by name).
        let (name, price) = if modifier_ids.is_empty() {
            (base_name, base_price)
        } else {
            let ph: Vec<String> = (0..modifier_ids.len()).map(|i| format!("?{}", i + 2)).collect();
            let sql = format!(
                "select name, price_delta from modifiers where product_id = ?1 and modifier_id in ({}) order by name",
                ph.join(",")
            );
            let mut args: Vec<String> = Vec::with_capacity(1 + modifier_ids.len());
            args.push(product_id.to_string());
            args.extend(modifier_ids.iter().cloned());
            let mut stmt = self.conn.prepare(&sql)?;
            let mods: Vec<(String, i64)> = stmt
                .query_map(params_from_iter(args.iter()), |r| Ok((r.get(0)?, r.get(1)?)))?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            if mods.is_empty() {
                (base_name, base_price)
            } else {
                let names = mods.iter().map(|m| m.0.as_str()).collect::<Vec<_>>().join(", ");
                (
                    format!("{} · {}", base_name, names),
                    base_price + mods.iter().map(|m| m.1).sum::<i64>(),
                )
            }
        };
        let server: String =
            self.conn
                .query_row("select server_id from checks where check_id = ?1", params![check_id], |r| {
                    r.get(0)
                })?;
        let existing: Option<String> = self
            .conn
            .query_row(
                "select item_id from order_items where check_id = ?1 and name = ?2 and state = 'HELD' order by created_at limit 1",
                params![check_id, name],
                |r| r.get(0),
            )
            .optional()?;
        match existing {
            Some(item_id) => {
                self.conn.execute(
                    "update order_items set qty = qty + ?2 where item_id = ?1",
                    params![item_id, qty],
                )?;
            }
            None => {
                self.conn.execute(
                    &format!(
                        "insert into order_items (item_id, check_id, server_id, product_id, name, qty, unit_price, state, created_at) \
                         values (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'HELD', {NOW})"
                    ),
                    params![new_id("itm"), check_id, server, product_id, name, qty, price],
                )?;
            }
        }
        Ok(())
    }

    pub fn inc_item(&self, item_id: &str, delta: i64) -> rusqlite::Result<()> {
        let q: i64 = self
            .conn
            .query_row(
                "select qty from order_items where item_id = ?1 and state = 'HELD'",
                params![item_id],
                |r| r.get(0),
            )
            .optional()?
            .unwrap_or(0);
        let nq = q + delta;
        if nq <= 0 {
            self.conn
                .execute("delete from order_items where item_id = ?1 and state = 'HELD'", params![item_id])?;
        } else {
            self.conn.execute(
                "update order_items set qty = ?2 where item_id = ?1 and state = 'HELD'",
                params![item_id, nq],
            )?;
        }
        Ok(())
    }

    pub fn send(&self, check_id: &str) -> rusqlite::Result<()> {
        let n = self.conn.execute(
            "update order_items set state = 'SENT' where check_id = ?1 and state = 'HELD'",
            params![check_id],
        )?;
        if n > 0 {
            self.conn
                .execute("update checks set status = 'IN_PROGRESS' where check_id = ?1", params![check_id])?;
        }
        Ok(())
    }

    pub fn void_item(&self, item_id: &str, reason_id: &str) -> rusqlite::Result<()> {
        self.conn.execute(
            "update order_items set state = 'VOID', reason_id = ?2 where item_id = ?1",
            params![item_id, reason_id],
        )?;
        Ok(())
    }

    pub fn comp_item(&self, item_id: &str, reason_id: &str) -> rusqlite::Result<()> {
        self.conn.execute(
            "update order_items set state = 'COMP', reason_id = ?2 where item_id = ?1",
            params![item_id, reason_id],
        )?;
        Ok(())
    }

    pub fn pay(&self, check_id: &str, method: &str) -> rusqlite::Result<i64> {
        let due: i64 = self.conn.query_row(
            "select coalesce(sum(qty*unit_price),0) from order_items where check_id = ?1 and state in ('HELD','SENT')",
            params![check_id],
            |r| r.get(0),
        )?;
        self.conn.execute(
            &format!("insert into payments (payment_id, check_id, method, amount, paid_at) values (?1, ?2, ?3, ?4, {NOW})"),
            params![new_id("pay"), check_id, method, due],
        )?;
        self.conn.execute(
            &format!("update checks set status = 'CLOSED_PAID', closed_at = {NOW} where check_id = ?1"),
            params![check_id],
        )?;
        let ticket: i64 =
            self.conn
                .query_row("select ticket_number from checks where check_id = ?1", params![check_id], |r| {
                    r.get(0)
                })?;
        Ok(ticket)
    }

    pub fn close_unpaid(&self, check_id: &str, reason_id: &str) -> rusqlite::Result<()> {
        self.conn.execute(
            &format!("update checks set status = 'CLOSED_UNPAID', reason_id = ?2, closed_at = {NOW} where check_id = ?1"),
            params![check_id, reason_id],
        )?;
        Ok(())
    }

    // ---- admin: settings ----
    pub fn update_settings(&self, spot: &str, currency: &str) -> Result<(), String> {
        let spot = spot.trim();
        let currency = currency.trim();
        if spot.is_empty() || currency.is_empty() {
            return Err("Libellé et devise requis".into());
        }
        self.conn
            .execute("update settings set spot_label=?1, currency_symbol=?2 where id=1", params![spot, currency])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    // ---- admin: servers ----
    pub fn all_servers(&self) -> rusqlite::Result<Vec<AdminServer>> {
        self.query_vec("select server_id, name, active from servers order by name", [], |r| {
            Ok(AdminServer { id: r.get(0)?, name: r.get(1)?, active: r.get::<_, i64>(2)? != 0 })
        })
    }

    pub fn create_server(&self, name: &str) -> Result<(), String> {
        self.conn
            .execute("insert into servers (server_id, name, active) values (?1,?2,1)", params![new_id("srv"), name.trim()])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn upsert_server(&self, id: &str, name: &str, active: bool) -> Result<(), String> {
        let name = name.trim();
        if name.is_empty() {
            return Err("Nom requis".into());
        }
        self.conn
            .execute(
                "insert into servers (server_id, name, active) values (?1,?2,?3) \
                 on conflict(server_id) do update set name=excluded.name, active=excluded.active",
                params![id, name, if active { 1 } else { 0 }],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    // ---- admin: users ----
    pub fn list_users(&self) -> rusqlite::Result<Vec<User>> {
        self.query_vec("select user_id, name, role, active from users order by name", [], |r| {
            Ok(User { id: r.get(0)?, name: r.get(1)?, role: r.get(2)?, active: r.get::<_, i64>(3)? != 0 })
        })
    }

    pub fn create_user(&self, name: &str, role: &str, pin: &str) -> Result<(), String> {
        let name = name.trim();
        if name.is_empty() {
            return Err("Nom requis".into());
        }
        if role != "cashier" && role != "admin" {
            return Err("Rôle invalide".into());
        }
        if pin.len() < 4 {
            return Err("Le code doit faire au moins 4 chiffres".into());
        }
        let hash = sha256_hex(pin);
        let dup: i64 = self
            .conn
            .query_row("select count(*) from users where pin_hash=?1 and active=1", params![hash], |r| r.get(0))
            .map_err(|e| e.to_string())?;
        if dup > 0 {
            return Err("Ce code est déjà utilisé".into());
        }
        self.conn
            .execute(
                "insert into users (user_id, name, pin_hash, role, active) values (?1,?2,?3,?4,1)",
                params![new_id("usr"), name, hash, role],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn update_user(&self, user_id: &str, name: &str, role: &str, active: bool) -> Result<(), String> {
        let name = name.trim();
        if name.is_empty() {
            return Err("Nom requis".into());
        }
        if role != "cashier" && role != "admin" {
            return Err("Rôle invalide".into());
        }
        // last-admin guard: block removing the final active admin
        if role != "admin" || !active {
            let n: i64 = self
                .conn
                .query_row(
                    "select count(*) from users where role='admin' and active=1 and user_id<>?1",
                    params![user_id],
                    |r| r.get(0),
                )
                .map_err(|e| e.to_string())?;
            if n == 0 {
                return Err("Au moins un administrateur actif est requis".into());
            }
        }
        self.conn
            .execute(
                "update users set name=?2, role=?3, active=?4 where user_id=?1",
                params![user_id, name, role, if active { 1 } else { 0 }],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn set_user_pin(&self, user_id: &str, pin: &str) -> Result<(), String> {
        if pin.len() < 4 {
            return Err("Le code doit faire au moins 4 chiffres".into());
        }
        let hash = sha256_hex(pin);
        let dup: i64 = self
            .conn
            .query_row(
                "select count(*) from users where pin_hash=?1 and active=1 and user_id<>?2",
                params![hash, user_id],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        if dup > 0 {
            return Err("Ce code est déjà utilisé".into());
        }
        self.conn
            .execute("update users set pin_hash=?2 where user_id=?1", params![user_id, hash])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn today(&self) -> String {
        self.conn.query_row("select date('now')", [], |r| r.get::<_, String>(0)).unwrap_or_default()
    }

    /// Prepare + map + collect in one shot (keeps the statement alive long enough).
    fn query_vec<T, P, F>(&self, sql: &str, params: P, f: F) -> rusqlite::Result<Vec<T>>
    where
        P: rusqlite::Params,
        F: FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<T>,
    {
        let mut stmt = self.conn.prepare(sql)?;
        let rows = stmt.query_map(params, f)?.collect::<rusqlite::Result<Vec<T>>>()?;
        Ok(rows)
    }

    pub fn global_report(&self) -> rusqlite::Result<GlobalReport> {
        let total_sales: i64 = self.conn.query_row(
            "select coalesce(sum(p.amount),0) from payments p join checks c on c.check_id=p.check_id where c.status='CLOSED_PAID'",
            [], |r| r.get(0))?;
        let paid_checks: i64 = self.conn.query_row("select count(*) from checks where status='CLOSED_PAID'", [], |r| r.get(0))?;
        let unpaid_checks: i64 = self.conn.query_row("select count(*) from checks where status='CLOSED_UNPAID'", [], |r| r.get(0))?;
        let by_zone = self.query_vec(
            "select z.name, coalesce(sum(p.amount),0) as a from checks c join payments p on p.check_id=c.check_id join zones z on z.zone_id=c.zone_id where c.status='CLOSED_PAID' group by z.name order by a desc",
            [], |r| Ok(Bar { label: r.get(0)?, amount: r.get(1)? }))?;
        let by_server = self.query_vec(
            "select s.name, coalesce(sum(p.amount),0) as a from checks c join payments p on p.check_id=c.check_id join servers s on s.server_id=c.server_id where c.status='CLOSED_PAID' group by s.name order by a desc",
            [], |r| Ok(Bar { label: r.get(0)?, amount: r.get(1)? }))?;
        let void_comp = self.query_vec(
            "select oi.state, coalesce(rc.label,'—'), sum(oi.qty), sum(oi.qty*oi.unit_price) from order_items oi left join reason_codes rc on rc.reason_id=oi.reason_id where oi.state in ('VOID','COMP') group by oi.state, rc.label",
            [], |r| Ok(VoidComp { state: r.get(0)?, label: r.get(1)?, count: r.get(2)?, amount: r.get(3)? }))?;
        Ok(GlobalReport { total_sales, paid_checks, unpaid_checks, by_zone, by_server, void_comp })
    }

    pub fn daily_report(&self, day: &str) -> rusqlite::Result<DailyReport> {
        let sales: i64 = self.conn.query_row(
            "select coalesce(sum(p.amount),0) from payments p join checks c on c.check_id=p.check_id where c.status='CLOSED_PAID' and date(c.opened_at)=?1",
            params![day], |r| r.get(0))?;
        let paid_count: i64 = self.conn.query_row(
            "select count(*) from checks where status='CLOSED_PAID' and date(opened_at)=?1", params![day], |r| r.get(0))?;
        let unpaid = self.query_vec(
            "select c.ticket_number, s.name, z.name, coalesce(rc.label,'—'), coalesce((select sum(oi.qty*oi.unit_price) from order_items oi where oi.check_id=c.check_id and oi.state in ('HELD','SENT')),0) from checks c join servers s on s.server_id=c.server_id join zones z on z.zone_id=c.zone_id left join reason_codes rc on rc.reason_id=c.reason_id where c.status='CLOSED_UNPAID' and date(c.opened_at)=?1 order by c.ticket_number",
            params![day], |r| Ok(Unpaid { ticket: r.get(0)?, server: r.get(1)?, zone: r.get(2)?, reason: r.get(3)?, amount: r.get(4)? }))?;
        let by_server = self.query_vec(
            "select s.name, coalesce(sum(p.amount),0) as a from checks c join payments p on p.check_id=c.check_id join servers s on s.server_id=c.server_id where c.status='CLOSED_PAID' and date(c.opened_at)=?1 group by s.name order by a desc",
            params![day], |r| Ok(Bar { label: r.get(0)?, amount: r.get(1)? }))?;
        let void_comp = self.query_vec(
            "select oi.state, coalesce(rc.label,'—'), sum(oi.qty), sum(oi.qty*oi.unit_price) from order_items oi join checks c on c.check_id=oi.check_id left join reason_codes rc on rc.reason_id=oi.reason_id where oi.state in ('VOID','COMP') and date(c.opened_at)=?1 group by oi.state, rc.label",
            params![day], |r| Ok(VoidComp { state: r.get(0)?, label: r.get(1)?, count: r.get(2)?, amount: r.get(3)? }))?;
        let unpaid_count = unpaid.len() as i64;
        Ok(DailyReport { date: day.to_string(), sales, paid_count, unpaid_count, unpaid, by_server, void_comp })
    }

    pub fn load_check(&self, check_id: &str) -> rusqlite::Result<CheckData> {
        let (ticket, status, zone_id, server_id, table_id, table_label): (
            i64,
            String,
            String,
            String,
            Option<String>,
            Option<String>,
        ) = self.conn.query_row(
            "select ticket_number, status, zone_id, server_id, table_id, table_label from checks where check_id = ?1",
            params![check_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?)),
        )?;
        let mut stmt = self.conn.prepare(
            "select item_id, name, qty, unit_price, state from order_items where check_id = ?1 order by created_at",
        )?;
        let items = stmt
            .query_map(params![check_id], |r| {
                Ok(CheckItem {
                    item_id: r.get(0)?,
                    name: r.get(1)?,
                    qty: r.get(2)?,
                    unit_price: r.get(3)?,
                    state: r.get(4)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(CheckData {
            check_id: check_id.to_string(),
            ticket_number: ticket,
            status,
            zone_id,
            server_id,
            table_id,
            table_label,
            items,
        })
    }
}

fn send_to_printer(s: &str, filename: &str) -> std::io::Result<()> {
    let dir = std::env::var("TEMP").unwrap_or_else(|_| ".".into());
    let path = PathBuf::from(dir).join(filename);
    std::fs::write(&path, s)?;
    #[cfg(windows)]
    {
        let _ = std::process::Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!("Get-Content -LiteralPath '{}' | Out-Printer", path.display()),
            ])
            .spawn();
    }
    Ok(())
}

/// Printable end-of-day summary (Clôturer la journée). ASCII for printer safety.
pub fn print_daily(r: &DailyReport, cur: &str) -> std::io::Result<()> {
    let mut s = String::new();
    s.push_str("           CAFE ADALYA\n      CLOTURE DE JOURNEE\n");
    s.push_str(&format!("           {}\n", r.date));
    s.push_str("--------------------------------\n");
    s.push_str(&format!("{:<16}{:>16}\n", "Ventes", format!("{} {}", r.sales, cur)));
    s.push_str(&format!("{:<16}{:>16}\n", "Notes payees", r.paid_count));
    s.push_str(&format!("{:<16}{:>16}\n", "Impayes", r.unpaid_count));
    s.push_str("--------------------------------\n");
    s.push_str("Ventes par serveur\n");
    if r.by_server.is_empty() {
        s.push_str("  (aucune)\n");
    }
    for b in &r.by_server {
        s.push_str(&format!("  {:<18}{:>10}\n", b.label, b.amount));
    }
    if !r.unpaid.is_empty() {
        s.push_str("--------------------------------\n");
        s.push_str("Impayes a recuperer\n");
        for u in &r.unpaid {
            s.push_str(&format!("  #{} {:<12}{:>8}\n    {}\n", u.ticket, u.server, u.amount, u.reason));
        }
    }
    s.push_str("--------------------------------\n");
    s.push_str("Signature: ____________________\n");
    send_to_printer(&s, "cloture_adalya.txt")
}

/// Build the facture text, write it to %TEMP%, and send it to the default
/// printer via PowerShell (Windows only). POC-level formatting.
pub fn print_facture(check: &CheckData) -> std::io::Result<()> {
    let mut s = String::new();
    s.push_str("           CAFE ADALYA\n             FACTURE\n");
    s.push_str(&format!("         Ticket no {}\n", check.ticket_number));
    s.push_str("--------------------------------\n");
    let mut total = 0i64;
    for it in &check.items {
        if it.state == "HELD" || it.state == "SENT" {
            let line = it.qty * it.unit_price;
            total += line;
            s.push_str(&format!(
                "{:<24}{:>6}\n",
                format!("{} x {}", it.qty, it.name),
                line
            ));
        }
    }
    s.push_str("--------------------------------\n");
    s.push_str(&format!("{:<24}{:>6}\n", "TOTAL (MRU)", total));
    s.push_str("\n      Merci de votre visite !\n");
    send_to_printer(&s, "facture_adalya.txt")
}
