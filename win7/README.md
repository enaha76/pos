# Caisse — native Windows 7 build (Slint)

A **native desktop** build of Caisse for **old Windows 7 machines with no browser**.
Unlike the Tauri build (`../src-tauri/`, which needs WebView2 / Windows 10+), this
one draws its own window with **Slint's software renderer** — no GPU, no WebView2,
no browser. Data is in **embedded SQLite** (`rusqlite`, bundled).

It reuses the exact same schema, seed and menu as the Tauri build
(`../src-tauri/migrations/*.sql`), so the menu and logic stay in sync.

## Status — proof of concept
Scope so far: **login → take an order → pay → print facture**, on the real
Café Adalya menu. This exists to prove the stack runs on the target Win7 POS
before porting the remaining screens (modifiers, zones/tables, reports, admin).

## Why Rust 1.77
Rust 1.78+ dropped Windows 7 from its standard library. This crate pins
**Rust 1.77.2** (`rust-toolchain.toml`) on the standard `x86_64-pc-windows-msvc`
target, which still produces Win7-compatible binaries.

## Build
```bash
# on Windows, from this folder:
cargo build --release
# → target/release/caisse-win7.exe
```
Or let GitHub Actions build it: push to `main` (workflow `win7.yml`) and download
the **`caisse-win7`** artifact.

## Data & printing
- Database: `%APPDATA%\CafeAdalyaCaisse\caisse.db` (created on first run).
- Default logins: **Caissier 1111**, **Propriétaire 9999** (hashed on first run).
- Facture printing (POC): writes `%TEMP%\facture_adalya.txt` and sends it to the
  default printer via PowerShell `Out-Printer`. A thermal/ESC-POS path can replace
  this once the client's printer is known.
