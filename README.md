# Cashier Charge

A touch-first restaurant **cashier / order & billing** app — an implementation of the two
specifications in [`docs/`](docs):

- `docs/cashier-charge.pdf` — Architecture & Business Logic
- `docs/uiux-spec.pdf` — UI/UX (CosyPOS visual direction)
- `docs/DECISIONS.md` — the resolved open questions baked into this build

## Stack

- **Vite + React + TypeScript** — client-first single-page app
- **Tailwind CSS v4** — CosyPOS design tokens live in `src/index.css`
- **Zustand** (`src/store/useStore.ts`) — the open-check model, order-item state
  machine, and audit trail, persisted to `localStorage` for offline resilience
- **vite-plugin-pwa** — installable on a landscape tablet

## Run

The frontend needs the backend running (Postgres is the source of truth).

```bash
# 1. backend (Rust + Postgres) — from the repo root
docker compose up -d            # API on http://localhost:8080

# 2. frontend
npm install
npm run dev                     # http://localhost:5173
npm run build                   # typecheck + production build (PWA)
```

Point at a different API with `VITE_API_URL` (defaults to `http://localhost:8080`).
Log in with a demo PIN: **Amina 1111**, **Bilal 2222**, **Cara 3333**.

## What's implemented

- **Cashier order screen** (three-panel: nav · menu · check) — zone tabs, table picker
  (none / free / fixed per zone), color-coded category chips + product grid
- **Open-check model** — ticket numbers, `OPEN → IN_PROGRESS → CLOSED_PAID/UNPAID`
- **Order-item state machine** — Held → Sent, with Void (pre-prep) and Comp (post-prep),
  each requiring an owner-managed reason; voids/comps are logged (no live approval — by decision)
- **Modifiers** — products with options open a picker; grouped options are single-select
  (e.g. steak temperature), ungrouped are multi-select add-ons; choices fold into the
  line's snapshot name + price
- **Payment** — cashier settles; check closes and rolls to the next ticket
- **Server attribution** — every check is assigned to a floor server (picker in the check
  header); the picker is filtered to the **roster** for that zone + current shift
- **Setup** — zones (add / rename / reorder / retire), products & categories (pastel-only
  color picker), and **servers & shifts** (roster: who works which zone per shift, per day),
  all data-driven
- **Reports** — sales by zone/server, void & comp by reason, unpaid checks, outlier flags
- **Settings** — configurable spot label, currency, demo reset

## Data layer

The app is wired to the Rust/Postgres backend in [`backend/`](backend). `useStore`
(Zustand) bootstraps config from `/api/config`, loads open checks from
`/api/checks/open`, and every cashier/config action calls the API (`src/lib/api.ts`),
then refreshes from the server. Postgres is the source of truth — no localStorage.
Money is stored in minor units (cents); product prices are snapshotted onto each order
line at entry time so menu changes never alter past bills.

**Accounts & roles.** Login (`POST /api/login`) authenticates a **user** (Caissier 1111 /
Propriétaire 9999) against a **bcrypt** hash; PINs are never plaintext or sent to clients.
Two roles: **cashier** (order screen only) and **admin** (also Configuration, Reports,
Settings). The role is enforced **server-side** on admin routes (via an `X-User-Id` header
checked against the DB), not just hidden in the UI. Admins manage accounts in-app
(**Configuration → Utilisateurs**): create users, set role, reset PIN — with duplicate-PIN
and last-admin lockout guards. The seed PINs are just a starting point. **Servers** (floor staff: Amina, Bilal,
Cara) have **no account** — they exist only for attribution and the roster; the cashier
picks the responsible server per note.

**Realtime.** Stations stay in sync via a WebSocket (`/ws`) — any change on one station
refetches on the others automatically.

## Known follow-ups

- Token/session-based auth (the role check currently trusts an `X-User-Id` header — fine on
  a trusted LAN, but a production deployment should use signed sessions/JWT)
- Offline buffering (the app now requires the backend)
- Split / partial payments and tips
