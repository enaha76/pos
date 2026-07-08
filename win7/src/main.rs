// Native Windows 7 build of Caisse (Slint + embedded SQLite, no browser/WebView2).
// Console stays visible for now so first-run errors are easy to see on the POS.
mod db;

use db::Db;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

slint::include_modules!();

/// Accent-name (from the seed/menu) → RGB, matching the web app's palette.
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
        price: 0,
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
        .map(|(id, label)| ReasonItem {
            id: id.into(),
            label: label.into(),
        })
        .collect();
    Rc::new(slint::VecModel::from(rows)).into()
}

/// Load the current check (or an empty state) into the UI.
fn refresh_check(ui: &MainWindow, db: &Db, current: &Option<String>) {
    ui.set_selected_item("".into());
    ui.set_selected_state("".into());
    match current {
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
                            total: line as i32,
                            state: it.state.clone().into(),
                        }
                    })
                    .collect();
                ui.set_lines(Rc::new(slint::VecModel::from(rows)).into());
                ui.set_total(subtotal as i32);
                ui.set_ticket(check.ticket_number as i32);
                ui.set_has_check(true);
                ui.set_can_send(held > 0);
                ui.set_can_pay(sent > 0 && held == 0);
            }
            Err(_) => set_empty(ui),
        },
        None => set_empty(ui),
    }
}

fn set_empty(ui: &MainWindow) {
    ui.set_lines(Rc::new(slint::VecModel::from(Vec::<CheckLine>::new())).into());
    ui.set_total(0);
    ui.set_ticket(0);
    ui.set_has_check(false);
    ui.set_can_send(false);
    ui.set_can_pay(false);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = Rc::new(Db::open()?);
    let ui = MainWindow::new()?;

    // reason codes
    ui.set_void_reasons(reason_model(&db, "void"));
    ui.set_comp_reasons(reason_model(&db, "comp"));
    ui.set_unpaid_reasons(reason_model(&db, "unpaid"));

    // categories + colour lookup
    let cats = db.categories()?;
    let mut cat_color: HashMap<String, slint::Color> = HashMap::new();
    let cat_items: Vec<CategoryChip> = cats
        .iter()
        .map(|c| {
            let col = accent_color(&c.color);
            cat_color.insert(c.id.clone(), col);
            CategoryChip {
                id: c.id.clone().into(),
                name: c.name.clone().into(),
                color: col,
            }
        })
        .collect();
    ui.set_categories(Rc::new(slint::VecModel::from(cat_items)).into());

    // products (all, with colour + category)
    let products = db.products()?;
    let all: Vec<(String, ProductItem)> = products
        .iter()
        .map(|p| {
            let col = cat_color
                .get(&p.category_id)
                .copied()
                .unwrap_or_else(|| accent_color("blue"));
            (
                p.category_id.clone(),
                ProductItem {
                    id: p.id.clone().into(),
                    name: p.name.clone().into(),
                    price: p.price as i32,
                    color: col,
                },
            )
        })
        .collect();
    let all = Rc::new(all);
    ui.set_grid(Rc::new(slint::VecModel::from(make_grid(
        &all.iter().map(|(_, it)| it.clone()).collect::<Vec<_>>(),
    ))).into());

    let current: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));

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

    // ---- login ----
    {
        let w = ui.as_weak();
        let db = db.clone();
        ui.on_login(move |pin| {
            let ui = w.unwrap();
            match db.login(pin.as_str()) {
                Ok(Some(u)) => {
                    ui.set_operator_name(u.name.into());
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
        let current = current.clone();
        ui.on_add_product(move |id| {
            let ui = w.unwrap();
            let cid = {
                let mut cur = current.borrow_mut();
                if cur.is_none() {
                    match db.create_check() {
                        Ok(c) => *cur = Some(c),
                        Err(e) => {
                            ui.set_status(format!("Erreur : {e}").into());
                            return;
                        }
                    }
                }
                cur.clone().unwrap()
            };
            if let Err(e) = db.add_item(&cid, id.as_str(), 1) {
                ui.set_status(format!("Erreur : {e}").into());
                return;
            }
            refresh_check(&ui, &db, &Some(cid));
            ui.set_status("".into());
        });
    }

    // ---- quantity +/- ----
    {
        let w = ui.as_weak();
        let db = db.clone();
        let current = current.clone();
        ui.on_inc_line(move |item_id, delta| {
            let ui = w.unwrap();
            if db.inc_item(item_id.as_str(), delta as i64).is_err() {
                return;
            }
            let cur = current.borrow().clone();
            refresh_check(&ui, &db, &cur);
            // keep the line selected so the user can keep adjusting
            ui.set_selected_item(item_id);
            ui.set_selected_state("HELD".into());
        });
    }

    // ---- send to kitchen ----
    {
        let w = ui.as_weak();
        let db = db.clone();
        let current = current.clone();
        ui.on_send(move || {
            let ui = w.unwrap();
            let cur = current.borrow().clone();
            if let Some(cid) = &cur {
                let _ = db.send(cid);
                ui.set_status("Articles envoyés en cuisine".into());
            }
            refresh_check(&ui, &db, &cur);
        });
    }

    // ---- void / comp / close-unpaid (via reason modal) ----
    {
        let w = ui.as_weak();
        let db = db.clone();
        let current = current.clone();
        ui.on_apply_reason(move |kind, item_id, reason_id| {
            let ui = w.unwrap();
            let cur = current.borrow().clone();
            match kind.as_str() {
                "void" => {
                    let _ = db.void_item(item_id.as_str(), reason_id.as_str());
                    ui.set_status("Article annulé".into());
                    refresh_check(&ui, &db, &cur);
                }
                "comp" => {
                    let _ = db.comp_item(item_id.as_str(), reason_id.as_str());
                    ui.set_status("Article offert".into());
                    refresh_check(&ui, &db, &cur);
                }
                "unpaid" => {
                    if let Some(cid) = &cur {
                        let _ = db.close_unpaid(cid, reason_id.as_str());
                    }
                    *current.borrow_mut() = None;
                    refresh_check(&ui, &db, &None);
                    ui.set_status("Note clôturée impayée".into());
                }
                _ => {}
            }
        });
    }

    // ---- print facture ----
    {
        let w = ui.as_weak();
        let db = db.clone();
        let current = current.clone();
        ui.on_print_facture(move || {
            let ui = w.unwrap();
            let cur = current.borrow().clone();
            match cur.and_then(|cid| db.load_check(&cid).ok()) {
                Some(check) => match db::print_facture(&check) {
                    Ok(_) => ui.set_status("Facture envoyée à l'impression".into()),
                    Err(e) => ui.set_status(format!("Impression : {e}").into()),
                },
                None => ui.set_status("Aucune note".into()),
            }
        });
    }

    // ---- pay ----
    {
        let w = ui.as_weak();
        let db = db.clone();
        let current = current.clone();
        ui.on_pay(move |method| {
            let ui = w.unwrap();
            let cur = current.borrow().clone();
            let Some(cid) = cur else {
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
                    *current.borrow_mut() = None;
                    refresh_check(&ui, &db, &None);
                    ui.set_status(format!("Payé ({method}) — ticket nº {ticket}").into());
                }
                Err(e) => ui.set_status(format!("Erreur : {e}").into()),
            }
        });
    }

    // ---- logout ----
    {
        let w = ui.as_weak();
        let db = db.clone();
        let current = current.clone();
        ui.on_logout(move || {
            let ui = w.unwrap();
            *current.borrow_mut() = None;
            refresh_check(&ui, &db, &None);
            ui.set_status("".into());
            ui.set_logged_in(false);
        });
    }

    ui.run()?;
    Ok(())
}
