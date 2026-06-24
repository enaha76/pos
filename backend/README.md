# Cashier Charge — Backend (Rust + Axum + Postgres)

A Dockerized REST API that is the source of truth for the Cashier Charge POS.

## Stack

- **Axum** (web) + **SQLx** (Postgres, runtime-checked queries)
- **Postgres 16** (relational, transactional — checks, payments, audit)
- Multi-stage Docker build → **distroless** runtime image (~20 MB)
- Migrations embedded at compile time and applied on startup

## Run

From the repo root:

```bash
docker compose up --build
```

This starts Postgres and the API (`http://localhost:8080`). The schema and seed
data are applied automatically on first boot.

## Endpoints

| Method | Path | Purpose |
|--------|------|---------|
| GET  | `/health` | liveness |
| POST | `/api/login` | validate a PIN server-side `{pin}` → `{server_id, name}` (401 on bad PIN) |
| GET  | `/ws` | WebSocket change feed — pushes `"checks"` / `"config"` on every mutation |
| GET  | `/api/config` | all owner config (zones, products, servers, shifts, roster, …) |
| GET  | `/api/checks/open` | open checks with their lines + payments |
| POST | `/api/checks` | open a check `{zone_id, server_id, table_id?, table_label?}` |
| POST | `/api/checks/add-item` | add a line `{check_id, product_id, qty?}` |
| POST | `/api/checks/send` | fire held lines `{check_id}` |
| POST | `/api/items/void` | void a line `{item_id, reason_id}` |
| POST | `/api/items/comp` | comp a line `{item_id, reason_id}` |
| POST | `/api/checks/pay` | settle `{check_id, method}` |
| POST | `/api/checks/close-unpaid` | walkout/write-off `{check_id, reason_id}` |
| GET  | `/api/reports/summary` | sales by zone/server, void/comp by reason |

Money is in integer minor units (cents). Prices are snapshotted onto each order
line at entry time so menu changes never alter past bills. Every money-affecting
action is written to `audit_log`.

## Notes

- The React frontend is wired to this API: it bootstraps from `/api/config`,
  loads open checks from `/api/checks/open`, and every cashier/config action
  calls these endpoints. The frontend no longer uses localStorage — Postgres is
  the source of truth. Point the frontend at a different API with
  `VITE_API_URL`.
- `servers.pin` is returned by `/api/config` for the demo; a real deployment
  would expose a `POST /api/login` instead and never send PINs to clients.
