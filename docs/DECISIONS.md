# Resolved Decisions

These close the "Open Questions to Confirm" from `cashier-charge.pdf` §11.

| # | Question | Decision | Notes |
|---|----------|----------|-------|
| 1 | Does the cashier also take payment? | **Yes — cashier settles** | Pay button on the cashier screen; no separate till role. |
| 2 | Can servers enter orders themselves? | **No — cashier-only** | `server_id` is an attribution field only; no server app. |
| 3 | When is manager approval needed for void/comp? | **Never in the moment** | No approval gate. Every void/comp is logged (server, reason, timestamp). Control = Reports + outlier alerts. |
| 4 | Target platform? | **Installable web app (PWA)** | Landscape tablet/terminal. Offline resilience via local persistence. |
| 5 | Word for the physical spot? | **Configurable label** | Global default in Settings; per-zone override (`zone.spot_label`). |

## Consequences folded into the build

- **Q3** has no real-time safeguard, so the Reports screen surfaces void/comp totals by reason and
  flags outlier servers prominently — that is the anti-theft control.
- **Q4 (PWA)** still needs a fuller offline/sync story before kitchen firing is production-grade.
  The current build persists all state locally (works offline) but does not yet sync across devices.
- **Q5** the label is dynamic everywhere it appears; the underlying `table_id` field is unchanged.
