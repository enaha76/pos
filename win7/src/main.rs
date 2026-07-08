// Native Windows 7 build of Caisse (Slint + embedded SQLite, no browser/WebView2).
// Console stays visible for now so first-run errors are easy to see on the POS.
mod db;

use db::{Db, SaleLine};
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

/// Chunk products into rows of 3 for the card grid.
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

fn set_grid(ui: &MainWindow, rows: Vec<GridRow>) {
    ui.set_grid(Rc::new(slint::VecModel::from(rows)).into());
}

fn refresh_check(ui: &MainWindow, lines: &[SaleLine]) {
    let rows: Vec<CheckLine> = lines
        .iter()
        .map(|l| CheckLine {
            name: l.name.clone().into(),
            qty: l.qty as i32,
            total: (l.qty * l.unit_price) as i32,
        })
        .collect();
    ui.set_lines(Rc::new(slint::VecModel::from(rows)).into());
    ui.set_total(lines.iter().map(|l| l.qty * l.unit_price).sum::<i64>() as i32);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = Rc::new(Db::open()?);
    let ui = MainWindow::new()?;

    // categories (+ colour lookup)
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

    // products: keep all (with colour + category) for filtering, plus a lookup map
    let products = db.products()?;
    let mut catalog: HashMap<String, (String, i64)> = HashMap::new();
    let all: Vec<(String, ProductItem)> = products
        .iter()
        .map(|p| {
            catalog.insert(p.id.clone(), (p.name.clone(), p.price));
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
    let catalog = Rc::new(catalog);

    set_grid(&ui, make_grid(&all.iter().map(|(_, it)| it.clone()).collect::<Vec<_>>()));

    let lines: Rc<RefCell<Vec<SaleLine>>> = Rc::new(RefCell::new(Vec::new()));

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
            set_grid(&ui, make_grid(&filtered));
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
        let lines = lines.clone();
        let catalog = catalog.clone();
        ui.on_add_product(move |id| {
            let ui = w.unwrap();
            let id = id.to_string();
            if let Some((name, price)) = catalog.get(&id) {
                {
                    let mut ls = lines.borrow_mut();
                    if let Some(l) = ls.iter_mut().find(|l| l.product_id == id) {
                        l.qty += 1;
                    } else {
                        ls.push(SaleLine {
                            product_id: id.clone(),
                            name: name.clone(),
                            qty: 1,
                            unit_price: *price,
                        });
                    }
                }
                refresh_check(&ui, &lines.borrow());
                ui.set_status("".into());
            }
        });
    }

    // ---- print facture ----
    {
        let w = ui.as_weak();
        let lines = lines.clone();
        ui.on_print_facture(move || {
            let ui = w.unwrap();
            let ls = lines.borrow();
            if ls.is_empty() {
                ui.set_status("Aucun article".into());
                return;
            }
            let total: i64 = ls.iter().map(|l| l.qty * l.unit_price).sum();
            match db::print_facture(&ls, total) {
                Ok(_) => ui.set_status("Facture envoyée à l'impression".into()),
                Err(e) => ui.set_status(format!("Impression : {e}").into()),
            }
        });
    }

    // ---- pay (with method) ----
    {
        let w = ui.as_weak();
        let lines = lines.clone();
        let db = db.clone();
        ui.on_pay(move |method| {
            let ui = w.unwrap();
            let total: i64;
            let result;
            {
                let ls = lines.borrow();
                if ls.is_empty() {
                    ui.set_status("Aucun article".into());
                    return;
                }
                total = ls.iter().map(|l| l.qty * l.unit_price).sum();
                result = db.record_sale(&ls, method.as_str(), total);
            }
            match result {
                Ok(ticket) => {
                    lines.borrow_mut().clear();
                    refresh_check(&ui, &lines.borrow());
                    ui.set_status(format!("Payé ({method}) — ticket nº {ticket}").into());
                }
                Err(e) => ui.set_status(format!("Erreur : {e}").into()),
            }
        });
    }

    // ---- logout ----
    {
        let w = ui.as_weak();
        let lines = lines.clone();
        ui.on_logout(move || {
            let ui = w.unwrap();
            lines.borrow_mut().clear();
            refresh_check(&ui, &lines.borrow());
            ui.set_status("".into());
            ui.set_logged_in(false);
        });
    }

    ui.run()?;
    Ok(())
}
