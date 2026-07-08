// Native Windows 7 build of Caisse (Slint + embedded SQLite, no browser/WebView2).
// Console stays visible for now so first-run errors are easy to see on the POS.
mod db;

use db::{Db, SaleLine};
use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;

slint::include_modules!();

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
    let total: i64 = lines.iter().map(|l| l.qty * l.unit_price).sum();
    ui.set_total(total as i32);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = Rc::new(Db::open()?);
    let ui = MainWindow::new()?;
    ui.set_currency("MRU".into());

    // catalog: model for the grid + lookup map for add-to-check
    let products = db.products()?;
    let mut catalog: HashMap<String, (String, i64)> = HashMap::new();
    let pmodel: Vec<ProductItem> = products
        .iter()
        .map(|p| {
            catalog.insert(p.id.clone(), (p.name.clone(), p.price));
            ProductItem {
                id: p.id.clone().into(),
                name: p.name.clone().into(),
                price: p.price as i32,
            }
        })
        .collect();
    ui.set_products(Rc::new(slint::VecModel::from(pmodel)).into());
    let catalog = Rc::new(catalog);

    // current check
    let lines: Rc<RefCell<Vec<SaleLine>>> = Rc::new(RefCell::new(Vec::new()));

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

    // ---- pay ----
    {
        let w = ui.as_weak();
        let lines = lines.clone();
        let db = db.clone();
        ui.on_pay(move || {
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
                result = db.record_sale(&ls, "Espèces", total);
            }
            match result {
                Ok(ticket) => {
                    lines.borrow_mut().clear();
                    refresh_check(&ui, &lines.borrow());
                    ui.set_status(format!("Payé — ticket nº {ticket}").into());
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
