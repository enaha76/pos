//! Embedded SQLite layer for the native Windows 7 build.
//! Reuses the exact same schema, seed and menu as the Tauri build
//! (../../src-tauri/migrations/*.sql) so the two stay in sync.

use rusqlite::{params, Connection};
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

pub struct UserRow {
    pub name: String,
    pub role: String,
}

/// One line on the current check (also used to build the facture).
pub struct SaleLine {
    pub product_id: String,
    pub name: String,
    pub qty: i64,
    pub unit_price: i64,
}

pub fn sha256_hex(s: &str) -> String {
    let mut h = Sha256::new();
    h.update(s.as_bytes());
    h.finalize().iter().map(|b| format!("{:02x}", b)).collect()
}

/// Per-user writable data dir: %APPDATA%\CafeAdalyaCaisse (fallback: current dir).
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

impl Db {
    pub fn open() -> rusqlite::Result<Self> {
        let conn = Connection::open(data_dir().join("caisse.db"))?;
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

    /// Seed PINs are plaintext; hash any that aren't already 64-hex SHA-256.
    fn hash_seed_pins(&self) -> rusqlite::Result<()> {
        let rows: Vec<(String, String)> = {
            let mut stmt = self.conn.prepare("select user_id, pin_hash from users")?;
            let it = stmt.query_map([], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
            })?;
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
            .prepare("select name, role from users where active = 1 and pin_hash = ?1 limit 1")?;
        let mut rows = stmt.query(params![h])?;
        match rows.next()? {
            Some(r) => Ok(Some(UserRow {
                name: r.get(0)?,
                role: r.get(1)?,
            })),
            None => Ok(None),
        }
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

    /// Persist a paid check (mirrors the Tauri `pay` flow). Returns the ticket number.
    pub fn record_sale(&self, lines: &[SaleLine], method: &str, total: i64) -> rusqlite::Result<i64> {
        let ticket: i64 =
            self.conn
                .query_row("select coalesce(max(ticket_number),0)+1 from checks", [], |r| {
                    r.get(0)
                })?;
        let zone: String =
            self.conn
                .query_row("select zone_id from zones order by display_order limit 1", [], |r| {
                    r.get(0)
                })?;
        let server: String =
            self.conn
                .query_row("select server_id from servers where active = 1 limit 1", [], |r| {
                    r.get(0)
                })?;
        let check_id = new_id("chk");
        self.conn.execute(
            "insert into checks (check_id, ticket_number, zone_id, server_id, status, opened_at, closed_at) \
             values (?1, ?2, ?3, ?4, 'CLOSED_PAID', strftime('%Y-%m-%dT%H:%M:%fZ','now'), strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            params![check_id, ticket, zone, server],
        )?;
        for l in lines {
            self.conn.execute(
                "insert into order_items (item_id, check_id, server_id, product_id, name, qty, unit_price, state, created_at) \
                 values (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'SENT', strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                params![new_id("itm"), check_id, server, l.product_id, l.name, l.qty, l.unit_price],
            )?;
        }
        self.conn.execute(
            "insert into payments (payment_id, check_id, method, amount, paid_at) \
             values (?1, ?2, ?3, ?4, strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            params![new_id("pay"), check_id, method, total],
        )?;
        Ok(ticket)
    }
}

/// Build the facture text, write it to %TEMP%, and send it to the default
/// printer via PowerShell (present on Windows 7). POC-level — a real thermal
/// (ESC/POS) path can replace this once the client's printer is known.
pub fn print_facture(lines: &[SaleLine], total: i64) -> std::io::Result<()> {
    let mut s = String::new();
    s.push_str("           CAFE ADALYA\n");
    s.push_str("             FACTURE\n");
    s.push_str("--------------------------------\n");
    for l in lines {
        s.push_str(&format!(
            "{:<24}{:>6}\n",
            format!("{} x {}", l.qty, l.name),
            l.qty * l.unit_price
        ));
    }
    s.push_str("--------------------------------\n");
    s.push_str(&format!("{:<24}{:>6}\n", "TOTAL (MRU)", total));
    s.push_str("\n      Merci de votre visite !\n");

    let dir = std::env::var("TEMP").unwrap_or_else(|_| ".".into());
    let path = PathBuf::from(dir).join("facture_adalya.txt");
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
