# Caisse — Windows desktop app (all-in-one)

A self-contained **Windows** build of Caisse using **Tauri**, with the data stored
in an **embedded SQLite** database (no Docker, no Postgres, no server). The whole
app — UI + data — runs from a single installed program, fully offline.

## How it works

- **Tauri** wraps the same React UI in a native window and produces a `.msi` / `.exe`.
- The data layer switches automatically (`src/lib/api.ts`):
  - **Desktop (Tauri):** embedded **SQLite** via `@tauri-apps/plugin-sql`
    (`src/lib/apiSqlite.ts`). The DB file lives in the app's data folder.
  - **Browser:** the HTTP backend in `backend/` (`src/lib/apiHttp.ts`).
- Schema + seed: `src-tauri/migrations/*.sql` (run automatically on first launch).
- PINs are seeded in plaintext and hashed to SHA-256 on first run. Demo logins:
  **Caissier 1111**, **Propriétaire 9999**.
- Realtime sync is disabled on desktop (single machine, no server).

## Build the installer

> Must be built **on Windows** (or via the included GitHub Actions workflow) —
> a Windows `.exe` can't be produced on Linux/macOS.

### Option A — GitHub Actions (no local setup)
Push to `main` (or run the **"Build Windows desktop app"** workflow manually).
Download the `caisse-windows` artifact — it contains the `.msi` and `.exe` installers.
See `.github/workflows/desktop.yml`.

### Option B — on a Windows PC
Prerequisites: Node 20+, Rust (https://rustup.rs), and the
[Microsoft C++ Build Tools] + WebView2 (preinstalled on Win 10/11).

```bash
npm ci
npm run tauri icon app-icon.png   # generate icons once (any 1024×1024 PNG)
npm run desktop:build             # outputs src-tauri/target/release/bundle/
```

The installer appears under `src-tauri/target/release/bundle/msi/` (and `nsis/`).

### Run in dev (live reload)
```bash
npm run desktop:dev
```

## Notes / follow-ups
- This build can't be compiled or tested on the Linux dev sandbox — only on
  Windows/CI — so expect a small amount of first-build iteration.
- Multi-station realtime is HTTP-backend only; the desktop build is single-PC.
- For automatic updates / code-signing, add the Tauri updater + a signing cert.
