// Native Windows 7 build of Caisse (Slint + embedded SQLite, no browser/WebView2).
// Release builds hide the console; panics are still written to a log file.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod db;

use db::{Db, NamedRow};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

slint::include_modules!();

// ---- draft / context ----

#[derive(Default, Clone)]
struct Draft {
    zone_id: Option<String>,
    table_id: Option<String>,
    table_label: Option<String>,
    server_id: Option<String>,
}

struct ZoneInfo {
    name: String,
    table_mode: String,
    spot_label: Option<String>,
}

/// Read-only lookups shared across callbacks.
struct Ctx {
    zone: HashMap<String, ZoneInfo>,
    table_label: HashMap<String, String>,
    server_name: HashMap<String, String>,
    spot_default: String,
    currency: String,
    shift_id: Option<String>,
}

fn accent_color(name: &str) -> slint::Color {
    let (r, g, b) = match name {
        "blue" => (0x25, 0x63, 0xeb),
        "pink" => (0xdb, 0x27, 0x77),
        "purple" => (0x7c, 0x3a, 0xed),
        "mint" => (0x16, 0xa3, 0x4a),
        "amber" => (0xd9, 0x77, 0x06),
        "coral" => (0xdc, 0x26, 0x26),
        _ => (0x25, 0x63, 0xeb),
    };
    slint::Color::from_rgb_u8(r, g, b)
}

fn make_grid(items: &[ProductItem]) -> Vec<GridRow> {
    let empty = ProductItem {
        id: Default::default(),
        name: Default::default(),
        price_label: Default::default(),
        color: slint::Color::from_argb_u8(0, 0, 0, 0),
    };
    let mut rows = Vec::new();
    for chunk in items.chunks(3) {
        rows.push(GridRow {
            a: chunk.first().cloned().unwrap_or_else(|| empty.clone()),
            b: chunk.get(1).cloned().unwrap_or_else(|| empty.clone()),
            c: chunk.get(2).cloned().unwrap_or_else(|| empty.clone()),
            count: chunk.len() as i32,
        });
    }
    rows
}

fn reason_model(db: &Db, kind: &str) -> slint::ModelRc<ReasonItem> {
    let rows: Vec<ReasonItem> = db
        .reason_codes(kind)
        .unwrap_or_default()
        .into_iter()
        .map(|(id, label)| ReasonItem { id: id.into(), label: label.into() })
        .collect();
    Rc::new(slint::VecModel::from(rows)).into()
}

/// Resolve the current shift by wall-clock (Mauritania is UTC+0, so UTC == local).
fn resolve_shift(shifts: &[(String, String, String)]) -> Option<String> {
    fn mins(t: &str) -> i64 {
        let mut p = t.split(':');
        let h: i64 = p.next().unwrap_or("0").parse().unwrap_or(0);
        let m: i64 = p.next().unwrap_or("0").parse().unwrap_or(0);
        h * 60 + m
    }
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let cur = ((secs % 86400) / 60) as i64;
    for (id, s, e) in shifts {
        let (s, e) = (mins(s), mins(e));
        let inside = if s <= e { cur >= s && cur < e } else { cur >= s || cur < e };
        if inside {
            return Some(id.clone());
        }
    }
    shifts.first().map(|(id, _, _)| id.clone())
}

/// Set the tables + servers models for a zone; returns a sensible default server id.
fn set_zone_models(ui: &MainWindow, db: &Db, zone_id: &str, shift_id: &Option<String>) -> Option<String> {
    let tables: Vec<TableSpot> = db
        .tables(zone_id)
        .unwrap_or_default()
        .into_iter()
        .map(|t| TableSpot { id: t.id.into(), label: t.label.into() })
        .collect();
    ui.set_tables(Rc::new(slint::VecModel::from(tables)).into());

    let rostered = shift_id
        .as_ref()
        .map(|s| db.rostered_servers(zone_id, s).unwrap_or_default())
        .unwrap_or_default();
    let all = db.servers().unwrap_or_default();
    let picker: Vec<NamedRow> = if rostered.is_empty() {
        all
    } else {
        all.into_iter().filter(|s| rostered.contains(&s.id)).collect()
    };
    let default_server = picker.first().map(|s| s.id.clone());
    let items: Vec<ServerItem> = picker
        .into_iter()
        .map(|s| ServerItem { id: s.id.into(), name: s.label.into() })
        .collect();
    ui.set_servers(Rc::new(slint::VecModel::from(items)).into());
    default_server
}

fn refresh(ui: &MainWindow, db: &Db, active: &Option<String>, draft: &Draft, ctx: &Ctx) {
    // open-checks bar
    let chips: Vec<CheckChip> = db
        .open_checks()
        .unwrap_or_default()
        .into_iter()
        .map(|c| {
            let tbl = if c.table_label.is_empty() {
                String::new()
            } else {
                format!(" · {}", c.table_label)
            };
            CheckChip {
                id: c.check_id.clone().into(),
                label: format!("nº{} {}{}", c.ticket_number, c.zone, tbl).into(),
                active: active.as_deref() == Some(c.check_id.as_str()),
            }
        })
        .collect();
    ui.set_open_checks(Rc::new(slint::VecModel::from(chips)).into());

    // header from the draft (kept in sync with the active check)
    let zinfo = draft.zone_id.as_ref().and_then(|z| ctx.zone.get(z));
    ui.set_zone_name(zinfo.map(|z| z.name.clone()).unwrap_or_default().into());
    ui.set_active_zone(draft.zone_id.clone().unwrap_or_default().into());
    ui.set_active_table_mode(zinfo.map(|z| z.table_mode.clone()).unwrap_or_else(|| "none".into()).into());
    let spot = zinfo
        .and_then(|z| z.spot_label.clone())
        .filter(|s| !s.is_empty()) // empty override must fall back, not blank the label
        .unwrap_or_else(|| ctx.spot_default.clone());
    ui.set_spot_label(spot.into());
    let tbl_label = draft
        .table_label
        .clone()
        .or_else(|| draft.table_id.as_ref().and_then(|t| ctx.table_label.get(t).cloned()))
        .unwrap_or_default();
    ui.set_table_label(tbl_label.into());
    ui.set_active_table_id(draft.table_id.clone().unwrap_or_default().into());
    let sname = draft
        .server_id
        .as_ref()
        .and_then(|s| ctx.server_name.get(s))
        .cloned()
        .unwrap_or_else(|| "Assigner…".into());
    ui.set_server_name(sname.into());

    // clear selection each refresh (kept explicitly by inc-line)
    ui.set_selected_item("".into());
    ui.set_selected_state("".into());

    // active check lines
    match active {
        Some(cid) => match db.load_check(cid) {
            Ok(check) => {
                let mut subtotal = 0i64;
                let mut held = 0i64;
                let mut sent = 0i64;
                let rows: Vec<CheckLine> = check
                    .items
                    .iter()
                    .map(|it| {
                        let line = it.qty * it.unit_price;
                        match it.state.as_str() {
                            "HELD" => {
                                subtotal += line;
                                held += it.qty;
                            }
                            "SENT" => {
                                subtotal += line;
                                sent += it.qty;
                            }
                            _ => {}
                        }
                        CheckLine {
                            item_id: it.item_id.clone().into(),
                            name: it.name.clone().into(),
                            qty: it.qty as i32,
                            total_label: money(line, &ctx.currency).into(),
                            state: it.state.clone().into(),
                        }
                    })
                    .collect();
                ui.set_lines(Rc::new(slint::VecModel::from(rows)).into());
                ui.set_total_label(money(subtotal, &ctx.currency).into());
                ui.set_ticket(check.ticket_number as i32);
                ui.set_has_check(true);
                ui.set_can_send(held > 0);
                ui.set_can_pay(sent > 0 && held == 0);
            }
            Err(_) => set_empty(ui, &ctx.currency),
        },
        None => set_empty(ui, &ctx.currency),
    }
}

fn set_empty(ui: &MainWindow, cur: &str) {
    ui.set_lines(Rc::new(slint::VecModel::from(Vec::<CheckLine>::new())).into());
    ui.set_total_label(money(0, cur).into());
    ui.set_ticket(0);
    ui.set_has_check(false);
    ui.set_can_send(false);
    ui.set_can_pay(false);
}

fn set_mods_model(ui: &MainWindow, mods: &[db::Modifier], selected: &[String]) {
    let rows: Vec<ModItem> = mods
        .iter()
        .map(|m| ModItem {
            id: m.id.clone().into(),
            name: m.name.clone().into(),
            note: if m.price_delta != 0 { format!("+{}", m.price_delta).into() } else { "".into() },
            selected: selected.iter().any(|s| s == &m.id),
        })
        .collect();
    ui.set_mods(Rc::new(slint::VecModel::from(rows)).into());
}

/// Format whole-MRU amounts with fr-FR thousands grouping, e.g. "2 800 MRU".
fn money(n: i64, cur: &str) -> String {
    let s = n.abs().to_string();
    let b = s.as_bytes();
    let mut out = String::new();
    for (i, c) in b.iter().enumerate() {
        if i > 0 && (b.len() - i) % 3 == 0 {
            out.push(' ');
        }
        out.push(*c as char);
    }
    format!("{}{} {}", if n < 0 { "-" } else { "" }, out, cur)
}

fn bars_model(bars: &[db::Bar], cur: &str) -> slint::ModelRc<BarItem> {
    let max = bars.iter().map(|b| b.amount).max().unwrap_or(0).max(1);
    let rows: Vec<BarItem> = bars
        .iter()
        .map(|b| BarItem {
            label: b.label.clone().into(),
            amount: money(b.amount, cur).into(),
            frac: (b.amount as f32 / max as f32),
        })
        .collect();
    Rc::new(slint::VecModel::from(rows)).into()
}

fn vc_model(vc: &[db::VoidComp], cur: &str) -> slint::ModelRc<VCItem> {
    let rows: Vec<VCItem> = vc
        .iter()
        .map(|v| VCItem {
            tag: if v.state == "VOID" { "Annulé".into() } else { "Offert".into() },
            label: v.label.clone().into(),
            count: v.count as i32,
            amount: money(v.amount, cur).into(),
        })
        .collect();
    Rc::new(slint::VecModel::from(rows)).into()
}

fn load_config(ui: &MainWindow, db: &Db) {
    let users: Vec<CfgUser> = db
        .list_users()
        .unwrap_or_default()
        .into_iter()
        .map(|u| CfgUser { id: u.id.into(), name: u.name.into(), role: u.role.into(), active: u.active })
        .collect();
    ui.set_cfg_users(Rc::new(slint::VecModel::from(users)).into());
    let servers: Vec<CfgServer> = db
        .all_servers()
        .unwrap_or_default()
        .into_iter()
        .map(|s| CfgServer { id: s.id.into(), name: s.name.into(), active: s.active })
        .collect();
    ui.set_cfg_servers(Rc::new(slint::VecModel::from(servers)).into());
    let prods: Vec<CfgProduct> = db
        .all_products()
        .unwrap_or_default()
        .into_iter()
        .map(|p| CfgProduct {
            id: p.id.into(),
            cat_id: p.category_id.into(),
            cat_name: p.category_name.into(),
            name: p.name.into(),
            price: p.price.to_string().into(),
            active: p.active,
        })
        .collect();
    ui.set_cfg_products(Rc::new(slint::VecModel::from(prods)).into());
    let zones: Vec<CfgZone> = db
        .all_zones()
        .unwrap_or_default()
        .into_iter()
        .map(|z| CfgZone { id: z.id.into(), name: z.name.into(), mode: z.mode.into(), spot: z.spot.into(), active: z.active })
        .collect();
    ui.set_cfg_zones(Rc::new(slint::VecModel::from(zones)).into());
}

fn load_reports(ui: &MainWindow, db: &Db, cur: &str) {
    let day = ui.get_report_date().to_string();
    let day = if day.is_empty() { db.today() } else { day };
    ui.set_report_date(day.clone().into());
    if let Ok(d) = db.daily_report(&day) {
        ui.set_rd_sales(money(d.sales, cur).into());
        ui.set_rd_paid(d.paid_count.to_string().into());
        ui.set_rd_unpaid(d.unpaid_count.to_string().into());
        let ul: Vec<UnpaidItem> = d
            .unpaid
            .iter()
            .map(|u| UnpaidItem {
                ticket: u.ticket as i32,
                zone: u.zone.clone().into(),
                server: u.server.clone().into(),
                reason: u.reason.clone().into(),
                amount: money(u.amount, cur).into(),
            })
            .collect();
        ui.set_rd_unpaid_list(Rc::new(slint::VecModel::from(ul)).into());
        ui.set_rd_by_server(bars_model(&d.by_server, cur));
        ui.set_rd_voidcomp(vc_model(&d.void_comp, cur));
    }
    if let Ok(g) = db.global_report() {
        ui.set_rg_sales(money(g.total_sales, cur).into());
        ui.set_rg_paid(g.paid_checks.to_string().into());
        ui.set_rg_unpaid(g.unpaid_checks.to_string().into());
        ui.set_rg_by_zone(bars_model(&g.by_zone, cur));
        ui.set_rg_by_server(bars_model(&g.by_server, cur));
        ui.set_rg_voidcomp(vc_model(&g.void_comp, cur));
    }
}

/// Ensure an open check exists (create from the draft on first item), then add.
fn ensure_and_add(
    db: &Db,
    active: &Rc<RefCell<Option<String>>>,
    draft: &Draft,
    product_id: &str,
    mods: &[String],
) -> Result<(), String> {
    let zone = draft.zone_id.clone().ok_or_else(|| "Choisissez une zone".to_string())?;
    let server = draft.server_id.clone().ok_or_else(|| "Choisissez un serveur".to_string())?;
    let cid = {
        let mut a = active.borrow_mut();
        if a.is_none() {
            let c = db
                .create_check(&zone, &server, draft.table_id.as_deref(), draft.table_label.as_deref())
                .map_err(|e| e.to_string())?;
            *a = Some(c);
        }
        a.clone().unwrap()
    };
    db.add_item(&cid, product_id, 1, mods).map_err(|e| e.to_string())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Console is hidden in release, so record panics to a log file for support.
    std::panic::set_hook(Box::new(|info| {
        let dir = std::path::PathBuf::from(std::env::var("APPDATA").unwrap_or_else(|_| ".".into()))
            .join("CafeAdalyaCaisse");
        let _ = std::fs::create_dir_all(&dir);
        let _ = std::fs::write(dir.join("panic.log"), format!("{info}\n"));
    }));

    let db = Rc::new(Db::open()?);
    let ui = MainWindow::new()?;

    // settings, reason codes
    let (spot_default, currency) = db.settings().unwrap_or_else(|_| ("Table".into(), "MRU".into()));
    ui.set_currency(currency.clone().into());
    ui.set_report_date(db.today().into());
    ui.set_cfg_spot(spot_default.clone().into());
    ui.set_cfg_currency(currency.clone().into());
    ui.set_cfg_prod_cat(db.first_category().unwrap_or_default().into());
    ui.set_void_reasons(reason_model(&db, "void"));
    ui.set_comp_reasons(reason_model(&db, "comp"));
    ui.set_unpaid_reasons(reason_model(&db, "unpaid"));

    // zones (+ lookup) and tabs
    let zones = db.zones()?;
    let mut zone_map: HashMap<String, ZoneInfo> = HashMap::new();
    let zone_tabs: Vec<ZoneTab> = zones
        .iter()
        .map(|z| {
            zone_map.insert(
                z.id.clone(),
                ZoneInfo {
                    name: z.name.clone(),
                    table_mode: z.table_mode.clone(),
                    spot_label: z.spot_label.clone(),
                },
            );
            ZoneTab { id: z.id.clone().into(), name: z.name.clone().into() }
        })
        .collect();
    ui.set_zones(Rc::new(slint::VecModel::from(zone_tabs)).into());

    // table + server name lookups
    let mut table_label: HashMap<String, String> = HashMap::new();
    for t in db.all_tables().unwrap_or_default() {
        table_label.insert(t.id, t.label);
    }
    let mut server_name: HashMap<String, String> = HashMap::new();
    let init_servers: Vec<ServerItem> = db
        .servers()
        .unwrap_or_default()
        .into_iter()
        .map(|s| {
            server_name.insert(s.id.clone(), s.label.clone());
            ServerItem { id: s.id.into(), name: s.label.into() }
        })
        .collect();
    ui.set_servers(Rc::new(slint::VecModel::from(init_servers)).into());

    let ctx = Rc::new(Ctx {
        zone: zone_map,
        table_label,
        server_name,
        spot_default,
        currency,
        shift_id: resolve_shift(&db.shifts().unwrap_or_default()),
    });

    // categories + colour lookup
    let cats = db.categories()?;
    let mut cat_color: HashMap<String, slint::Color> = HashMap::new();
    let cat_items: Vec<CategoryChip> = cats
        .iter()
        .map(|c| {
            let col = accent_color(&c.color);
            cat_color.insert(c.id.clone(), col);
            CategoryChip { id: c.id.clone().into(), name: c.name.clone().into(), color: col }
        })
        .collect();
    ui.set_categories(Rc::new(slint::VecModel::from(cat_items)).into());

    // products (all, with colour + category)
    let products = db.products()?;
    let all: Vec<(String, ProductItem)> = products
        .iter()
        .map(|p| {
            let col = cat_color.get(&p.category_id).copied().unwrap_or_else(|| accent_color("blue"));
            (
                p.category_id.clone(),
                ProductItem {
                    id: p.id.clone().into(),
                    name: p.name.clone().into(),
                    price_label: money(p.price, &ctx.currency).into(),
                    color: col,
                },
            )
        })
        .collect();
    let all = Rc::new(all);
    ui.set_grid(Rc::new(slint::VecModel::from(make_grid(
        &all.iter().map(|(_, it)| it.clone()).collect::<Vec<_>>(),
    ))).into());

    let active: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
    let draft: Rc<RefCell<Draft>> = Rc::new(RefCell::new(Draft::default()));
    // (user_id, role) captured at login — for role gating + audit later.
    let session: Rc<RefCell<(String, String)>> = Rc::new(RefCell::new((String::new(), String::new())));
    // modifier picker state
    let pending_product: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
    let pending_mods: Rc<RefCell<Vec<db::Modifier>>> = Rc::new(RefCell::new(Vec::new()));
    let selected_mods: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));

    // ---- filter by category ----
    {
        let w = ui.as_weak();
        let all = all.clone();
        ui.on_select_category(move |cat| {
            let ui = w.unwrap();
            let cat = cat.to_string();
            let filtered: Vec<ProductItem> = all
                .iter()
                .filter(|(cid, _)| cat == "all" || *cid == cat)
                .map(|(_, it)| it.clone())
                .collect();
            ui.set_grid(Rc::new(slint::VecModel::from(make_grid(&filtered))).into());
        });
    }

    // ---- select zone (starts a fresh draft) ----
    {
        let w = ui.as_weak();
        let db = db.clone();
        let ctx = ctx.clone();
        let active = active.clone();
        let draft = draft.clone();
        ui.on_select_zone(move |zid| {
            let ui = w.unwrap();
            let zid = zid.to_string();
            let default_server = set_zone_models(&ui, &db, &zid, &ctx.shift_id);
            {
                let mut d = draft.borrow_mut();
                d.zone_id = Some(zid);
                d.table_id = None;
                d.table_label = None;
                d.server_id = default_server;
            }
            *active.borrow_mut() = None;
            refresh(&ui, &db, &active.borrow(), &draft.borrow(), &ctx);
        });
    }

    // ---- set table ----
    {
        let w = ui.as_weak();
        let db = db.clone();
        let ctx = ctx.clone();
        let active = active.clone();
        let draft = draft.clone();
        ui.on_set_table(move |table_id, table_label| {
            let ui = w.unwrap();
            let tid = if table_id.is_empty() { None } else { Some(table_id.to_string()) };
            let tl = if table_label.is_empty() { None } else { Some(table_label.to_string()) };
            {
                let mut d = draft.borrow_mut();
                d.table_id = tid.clone();
                d.table_label = tl.clone();
            }
            let a = active.borrow().clone();
            if let Some(cid) = &a {
                let _ = db.set_check_table(cid, tid.as_deref(), tl.as_deref());
            }
            refresh(&ui, &db, &active.borrow(), &draft.borrow(), &ctx);
        });
    }

    // ---- pick server ----
    {
        let w = ui.as_weak();
        let db = db.clone();
        let ctx = ctx.clone();
        let active = active.clone();
        let draft = draft.clone();
        ui.on_pick_server(move |sid| {
            let ui = w.unwrap();
            let sid = sid.to_string();
            draft.borrow_mut().server_id = Some(sid.clone());
            let a = active.borrow().clone();
            if let Some(cid) = &a {
                let _ = db.set_check_server(cid, &sid);
            }
            refresh(&ui, &db, &active.borrow(), &draft.borrow(), &ctx);
        });
    }

    // ---- open an existing check ----
    {
        let w = ui.as_weak();
        let db = db.clone();
        let ctx = ctx.clone();
        let active = active.clone();
        let draft = draft.clone();
        ui.on_open_check(move |cid| {
            let ui = w.unwrap();
            let cid = cid.to_string();
            if let Ok(check) = db.load_check(&cid) {
                set_zone_models(&ui, &db, &check.zone_id, &ctx.shift_id);
                {
                    let mut d = draft.borrow_mut();
                    d.zone_id = Some(check.zone_id.clone());
                    d.server_id = Some(check.server_id.clone());
                    d.table_id = check.table_id.clone();
                    d.table_label = check.table_label.clone();
                }
                *active.borrow_mut() = Some(cid);
            }
            refresh(&ui, &db, &active.borrow(), &draft.borrow(), &ctx);
        });
    }

    // ---- new check (keep zone/server, clear table + active) ----
    {
        let w = ui.as_weak();
        let db = db.clone();
        let ctx = ctx.clone();
        let active = active.clone();
        let draft = draft.clone();
        ui.on_new_check(move || {
            let ui = w.unwrap();
            {
                let mut d = draft.borrow_mut();
                d.table_id = None;
                d.table_label = None;
            }
            *active.borrow_mut() = None;
            refresh(&ui, &db, &active.borrow(), &draft.borrow(), &ctx);
        });
    }

    // ---- login ----
    {
        let w = ui.as_weak();
        let db = db.clone();
        let session = session.clone();
        ui.on_login(move |pin| {
            let ui = w.unwrap();
            match db.login(pin.as_str()) {
                Ok(Some(u)) => {
                    *session.borrow_mut() = (u.user_id, u.role.clone());
                    ui.set_operator_name(u.name.into());
                    ui.set_operator_role(u.role.into());
                    ui.set_login_error("".into());
                    ui.set_status("".into());
                    ui.set_logged_in(true);
                }
                Ok(None) => ui.set_login_error("Code invalide".into()),
                Err(e) => ui.set_login_error(format!("Erreur : {e}").into()),
            }
        });
    }

    // ---- add product ----
    {
        let w = ui.as_weak();
        let db = db.clone();
        let ctx = ctx.clone();
        let active = active.clone();
        let draft = draft.clone();
        let pending_product = pending_product.clone();
        let pending_mods = pending_mods.clone();
        let selected_mods = selected_mods.clone();
        ui.on_add_product(move |id| {
            let ui = w.unwrap();
            let pid = id.to_string();
            // need a zone + server before starting a check
            {
                let d = draft.borrow();
                if d.zone_id.is_none() {
                    ui.set_status("Choisissez une zone".into());
                    return;
                }
                if d.server_id.is_none() {
                    ui.set_status("Choisissez un serveur".into());
                    return;
                }
            }
            // if the product has modifiers, choose them first
            let mods = db.modifiers(&pid).unwrap_or_default();
            if !mods.is_empty() {
                *pending_product.borrow_mut() = Some(pid);
                selected_mods.borrow_mut().clear();
                set_mods_model(&ui, &mods, &[]);
                *pending_mods.borrow_mut() = mods;
                ui.set_modal_kind("mods".into());
                return;
            }
            match ensure_and_add(&db, &active, &draft.borrow(), &pid, &[]) {
                Ok(()) => {
                    refresh(&ui, &db, &active.borrow(), &draft.borrow(), &ctx);
                    ui.set_status("".into());
                }
                Err(e) => ui.set_status(e.into()),
            }
        });
    }

    // ---- modifier picker: toggle (single-select within a group) ----
    {
        let w = ui.as_weak();
        let pending_mods = pending_mods.clone();
        let selected_mods = selected_mods.clone();
        ui.on_toggle_mod(move |id| {
            let ui = w.unwrap();
            let id = id.to_string();
            let mods = pending_mods.borrow();
            let group = mods.iter().find(|m| m.id == id).and_then(|m| m.mod_group.clone());
            {
                let mut sel = selected_mods.borrow_mut();
                if sel.iter().any(|s| s == &id) {
                    sel.retain(|s| s != &id);
                } else {
                    if let Some(g) = &group {
                        let same: Vec<String> = mods
                            .iter()
                            .filter(|m| m.mod_group.as_ref() == Some(g))
                            .map(|m| m.id.clone())
                            .collect();
                        sel.retain(|s| !same.contains(s));
                    }
                    sel.push(id);
                }
            }
            set_mods_model(&ui, &mods, &selected_mods.borrow());
        });
    }

    // ---- modifier picker: confirm ----
    {
        let w = ui.as_weak();
        let db = db.clone();
        let ctx = ctx.clone();
        let active = active.clone();
        let draft = draft.clone();
        let pending_product = pending_product.clone();
        let selected_mods = selected_mods.clone();
        ui.on_confirm_mods(move || {
            let ui = w.unwrap();
            let pid = pending_product.borrow().clone();
            ui.set_modal_kind("".into());
            let Some(pid) = pid else { return };
            let sel = selected_mods.borrow().clone();
            match ensure_and_add(&db, &active, &draft.borrow(), &pid, &sel) {
                Ok(()) => {
                    refresh(&ui, &db, &active.borrow(), &draft.borrow(), &ctx);
                    ui.set_status("".into());
                }
                Err(e) => ui.set_status(e.into()),
            }
        });
    }

    // ---- quantity +/- ----
    {
        let w = ui.as_weak();
        let db = db.clone();
        let ctx = ctx.clone();
        let active = active.clone();
        let draft = draft.clone();
        ui.on_inc_line(move |item_id, delta| {
            let ui = w.unwrap();
            let _ = db.inc_item(item_id.as_str(), delta as i64);
            refresh(&ui, &db, &active.borrow(), &draft.borrow(), &ctx);
            ui.set_selected_item(item_id);
            ui.set_selected_state("HELD".into());
        });
    }

    // ---- send ----
    {
        let w = ui.as_weak();
        let db = db.clone();
        let ctx = ctx.clone();
        let active = active.clone();
        let draft = draft.clone();
        ui.on_send(move || {
            let ui = w.unwrap();
            let a = active.borrow().clone();
            if let Some(cid) = &a {
                let _ = db.send(cid);
                ui.set_status("Articles envoyés en cuisine".into());
            }
            refresh(&ui, &db, &active.borrow(), &draft.borrow(), &ctx);
        });
    }

    // ---- void / comp / close-unpaid ----
    {
        let w = ui.as_weak();
        let db = db.clone();
        let ctx = ctx.clone();
        let active = active.clone();
        let draft = draft.clone();
        ui.on_apply_reason(move |kind, item_id, reason_id| {
            let ui = w.unwrap();
            match kind.as_str() {
                "void" => {
                    let _ = db.void_item(item_id.as_str(), reason_id.as_str());
                    ui.set_status("Article annulé".into());
                }
                "comp" => {
                    let _ = db.comp_item(item_id.as_str(), reason_id.as_str());
                    ui.set_status("Article offert".into());
                }
                "unpaid" => {
                    let a = active.borrow().clone();
                    if let Some(cid) = &a {
                        let _ = db.close_unpaid(cid, reason_id.as_str());
                    }
                    *active.borrow_mut() = None;
                    ui.set_status("Note clôturée impayée".into());
                }
                _ => {}
            }
            refresh(&ui, &db, &active.borrow(), &draft.borrow(), &ctx);
        });
    }

    // ---- print facture ----
    {
        let w = ui.as_weak();
        let db = db.clone();
        let ctx = ctx.clone();
        let active = active.clone();
        ui.on_print_facture(move || {
            let ui = w.unwrap();
            let a = active.borrow().clone();
            match a.and_then(|cid| db.load_check(&cid).ok()) {
                Some(check) => {
                    let zone = ctx.zone.get(&check.zone_id).map(|z| z.name.clone()).unwrap_or_default();
                    let table = check
                        .table_label
                        .clone()
                        .or_else(|| check.table_id.as_ref().and_then(|t| ctx.table_label.get(t).cloned()))
                        .unwrap_or_default();
                    let server = ctx.server_name.get(&check.server_id).cloned().unwrap_or_default();
                    let date = db.today();
                    match db::print_facture(&check, &zone, &table, &server, &date, &ctx.currency) {
                        Ok(path) => ui.set_status(format!("Facture prête : {path}").into()),
                        Err(e) => ui.set_status(format!("Impression : {e}").into()),
                    }
                }
                None => ui.set_status("Aucune note".into()),
            }
        });
    }

    // ---- pay ----
    {
        let w = ui.as_weak();
        let db = db.clone();
        let ctx = ctx.clone();
        let active = active.clone();
        let draft = draft.clone();
        ui.on_pay(move |method| {
            let ui = w.unwrap();
            let a = active.borrow().clone();
            let Some(cid) = a else {
                ui.set_status("Aucune note".into());
                return;
            };
            let check = match db.load_check(&cid) {
                Ok(c) => c,
                Err(e) => {
                    ui.set_status(format!("Erreur : {e}").into());
                    return;
                }
            };
            let held = check.items.iter().filter(|i| i.state == "HELD").count();
            let sent = check.items.iter().filter(|i| i.state == "SENT").count();
            if sent == 0 {
                ui.set_status("Rien à payer".into());
                return;
            }
            if held > 0 {
                ui.set_status("Envoyez d'abord les articles".into());
                return;
            }
            match db.pay(&cid, method.as_str()) {
                Ok(ticket) => {
                    *active.borrow_mut() = None;
                    refresh(&ui, &db, &active.borrow(), &draft.borrow(), &ctx);
                    ui.set_status(format!("Payé ({method}) — ticket nº {ticket}").into());
                }
                Err(e) => ui.set_status(format!("Erreur : {e}").into()),
            }
        });
    }

    // ---- refresh reports ----
    {
        let w = ui.as_weak();
        let db = db.clone();
        let cur = ctx.currency.clone();
        ui.on_refresh_reports(move || {
            let ui = w.unwrap();
            load_reports(&ui, &db, &cur);
        });
    }

    // ---- print end-of-day ----
    {
        let w = ui.as_weak();
        let db = db.clone();
        let cur = ctx.currency.clone();
        ui.on_print_daily(move || {
            let ui = w.unwrap();
            let day = ui.get_report_date().to_string();
            let day = if day.is_empty() { db.today() } else { day };
            match db.daily_report(&day) {
                Ok(d) => match db::print_daily(&d, &cur) {
                    Ok(path) => ui.set_status(format!("Clôture prête : {path}").into()),
                    Err(e) => ui.set_status(format!("Impression : {e}").into()),
                },
                Err(e) => ui.set_status(format!("Erreur : {e}").into()),
            }
        });
    }

    // ---- config: refresh lists ----
    {
        let w = ui.as_weak();
        let db = db.clone();
        ui.on_config_refresh(move || {
            let ui = w.unwrap();
            load_config(&ui, &db);
        });
    }

    // ---- config: users ----
    {
        let w = ui.as_weak();
        let db = db.clone();
        ui.on_save_user(move |id, name, role, active| {
            let ui = w.unwrap();
            match db.update_user(id.as_str(), name.as_str(), role.as_str(), active) {
                Ok(()) => {
                    ui.set_status("Utilisateur enregistré".into());
                    load_config(&ui, &db);
                }
                Err(e) => ui.set_status(e.into()),
            }
        });
    }
    {
        let w = ui.as_weak();
        let db = db.clone();
        ui.on_create_user(move || {
            let ui = w.unwrap();
            let name = ui.get_nu_name().to_string();
            let role = ui.get_nu_role().to_string();
            let pin = ui.get_nu_pin().to_string();
            match db.create_user(&name, &role, &pin) {
                Ok(()) => {
                    ui.set_nu_name("".into());
                    ui.set_nu_pin("".into());
                    ui.set_status("Utilisateur créé".into());
                    load_config(&ui, &db);
                }
                Err(e) => ui.set_status(e.into()),
            }
        });
    }
    {
        let w = ui.as_weak();
        let db = db.clone();
        ui.on_set_user_pin(move |id, pin| {
            let ui = w.unwrap();
            match db.set_user_pin(id.as_str(), pin.as_str()) {
                Ok(()) => {
                    ui.set_status("Code mis à jour".into());
                    load_config(&ui, &db);
                }
                Err(e) => ui.set_status(e.into()),
            }
        });
    }

    // ---- config: servers ----
    {
        let w = ui.as_weak();
        let db = db.clone();
        ui.on_save_server(move |id, name, active| {
            let ui = w.unwrap();
            match db.upsert_server(id.as_str(), name.as_str(), active) {
                Ok(()) => {
                    ui.set_status("Serveur enregistré".into());
                    load_config(&ui, &db);
                }
                Err(e) => ui.set_status(e.into()),
            }
        });
    }
    {
        let w = ui.as_weak();
        let db = db.clone();
        ui.on_add_server(move || {
            let ui = w.unwrap();
            match db.create_server("Nouveau serveur") {
                Ok(()) => {
                    ui.set_status("Serveur ajouté".into());
                    load_config(&ui, &db);
                }
                Err(e) => ui.set_status(e.into()),
            }
        });
    }

    // ---- config: products ----
    {
        let w = ui.as_weak();
        let db = db.clone();
        ui.on_save_product(move |id, cat_id, name, price, active| {
            let ui = w.unwrap();
            let price = match price.to_string().trim().parse::<i64>() {
                Ok(p) if p >= 0 => p,
                _ => {
                    ui.set_status("Prix invalide".into());
                    return;
                }
            };
            match db.upsert_product(id.as_str(), name.as_str(), cat_id.as_str(), price, active) {
                Ok(()) => {
                    ui.set_status("Produit enregistré (redémarrez pour l'écran de commande)".into());
                    load_config(&ui, &db);
                }
                Err(e) => ui.set_status(e.into()),
            }
        });
    }
    {
        let w = ui.as_weak();
        let db = db.clone();
        ui.on_create_product(move |cat_id| {
            let ui = w.unwrap();
            let cat = cat_id.to_string();
            let cat = if cat.is_empty() { db.first_category().unwrap_or_default() } else { cat };
            match db.create_product(&cat, "Nouveau produit") {
                Ok(()) => {
                    ui.set_status("Produit ajouté".into());
                    load_config(&ui, &db);
                }
                Err(e) => ui.set_status(e.into()),
            }
        });
    }

    // ---- config: zones ----
    {
        let w = ui.as_weak();
        let db = db.clone();
        ui.on_save_zone(move |id, name, mode, spot, active| {
            let ui = w.unwrap();
            let spot = spot.to_string();
            let spot_opt = if spot.trim().is_empty() { None } else { Some(spot.as_str()) };
            match db.upsert_zone(id.as_str(), name.as_str(), mode.as_str(), spot_opt, 0, active) {
                Ok(()) => {
                    ui.set_status("Zone enregistrée (redémarrez pour l'écran de commande)".into());
                    load_config(&ui, &db);
                }
                Err(e) => ui.set_status(e.into()),
            }
        });
    }
    {
        let w = ui.as_weak();
        let db = db.clone();
        ui.on_create_zone(move || {
            let ui = w.unwrap();
            match db.create_zone("Nouvelle zone") {
                Ok(()) => {
                    ui.set_status("Zone ajoutée".into());
                    load_config(&ui, &db);
                }
                Err(e) => ui.set_status(e.into()),
            }
        });
    }

    // ---- config: settings ----
    {
        let w = ui.as_weak();
        let db = db.clone();
        ui.on_save_settings(move || {
            let ui = w.unwrap();
            let spot = ui.get_cfg_spot().to_string();
            let cur = ui.get_cfg_currency().to_string();
            match db.update_settings(&spot, &cur) {
                Ok(()) => {
                    ui.set_currency(cur.into());
                    ui.set_status("Réglages enregistrés (redémarrez pour tout appliquer)".into());
                }
                Err(e) => ui.set_status(e.into()),
            }
        });
    }

    // ---- logout ----
    {
        let w = ui.as_weak();
        let db = db.clone();
        let ctx = ctx.clone();
        let active = active.clone();
        let draft = draft.clone();
        ui.on_logout(move || {
            let ui = w.unwrap();
            *active.borrow_mut() = None;
            *draft.borrow_mut() = Draft::default();
            refresh(&ui, &db, &active.borrow(), &draft.borrow(), &ctx);
            ui.set_status("".into());
            ui.set_route("order".into());
            ui.set_logged_in(false);
        });
    }

    ui.run()?;
    Ok(())
}
