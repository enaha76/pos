/*
  Domain model — Cashier Charge.
  Direct translation of the Architecture & Business Logic spec
  (docs/cashier-charge.pdf §9 Data Model), with the resolved
  open-question decisions folded in (see docs/DECISIONS.md).
*/

/** Accent tokens from the CosyPOS palette (uiux-spec §2.2). */
export type AccentColor = "blue" | "pink" | "purple" | "mint" | "amber" | "coral";

/** §4.1 — table-number behaviour, decided per zone. */
export type TableMode = "none" | "free" | "fixed";

/** §5.1 — the check lifecycle. */
export type CheckStatus =
  | "OPEN"
  | "IN_PROGRESS"
  | "CLOSED_PAID"
  | "CLOSED_UNPAID"
  | "VOIDED";

/** §5.2 — order-item state machine. */
export type OrderItemState = "HELD" | "SENT" | "VOID" | "COMP";

/** §8 — reason codes are typed by the action they justify. */
export type ReasonKind = "void" | "comp" | "unpaid";

export interface Zone {
  zone_id: string;
  name: string;
  display_order: number;
  table_mode: TableMode;
  active: boolean;
  /** Q5 — per-zone override of the spot label (else falls back to settings). */
  spot_label?: string;
}

/** Only used by fixed-mode zones (§9). */
export interface TableSpot {
  table_id: string;
  zone_id: string;
  label: string;
  active: boolean;
}

export interface Server {
  server_id: string;
  name: string;
  /** Never sent by the API — login is validated server-side via POST /api/login. */
  pin?: string;
  active: boolean;
}

/** A login account (operator). PINs are never returned by the API. */
export interface User {
  user_id: string;
  name: string;
  role: "cashier" | "admin";
  active: boolean;
}

export interface Shift {
  shift_id: string;
  name: string;
  start_time: string; // "07:00"
  end_time: string; // "16:00"
}

/** §9 — which server works which zone, per shift, per day. */
export interface ShiftAssignment {
  assignment_id: string;
  server_id: string;
  zone_id: string;
  shift_id: string;
  date: string; // YYYY-MM-DD
}

export interface Category {
  category_id: string;
  name: string;
  color: AccentColor;
  display_order: number;
}

export interface Product {
  product_id: string;
  name: string;
  category_id: string;
  price: number; // minor units (cents) to avoid float drift
  active: boolean;
}

export interface Modifier {
  modifier_id: string;
  product_id: string;
  name: string;
  price_delta: number;
  /** Options in the same group are single-select; null/undefined = add-on. */
  mod_group?: string | null;
}

export interface ReasonCode {
  reason_id: string;
  kind: ReasonKind;
  label: string;
  active: boolean;
}

/** §9 — prices are copied onto the line at entry time so menu changes never alter past bills. */
export interface OrderItem {
  item_id: string;
  check_id: string;
  server_id: string;
  product_id: string;
  name: string; // snapshot of product name
  qty: number;
  unit_price: number; // snapshot of price at entry (minor units)
  state: OrderItemState;
  reason_id?: string;
  created_at: string; // ISO timestamp from the API
}

export interface Payment {
  payment_id: string;
  check_id: string;
  method: string;
  amount: number;
  paid_at: string; // ISO timestamp from the API
}

export interface Check {
  check_id: string;
  ticket_number: number;
  zone_id: string;
  server_id: string;
  table_id?: string; // null when zone uses no tables or left blank (§9)
  table_label?: string; // free-text label for free-mode zones
  status: CheckStatus;
  opened_at: string; // ISO timestamp from the API
  closed_at?: string;
  items: OrderItem[];
  payments: Payment[];
  /** reason for a CLOSED_UNPAID / VOIDED check. */
  reason_id?: string;
}

export interface AuditEntry {
  log_id: string;
  actor_id: string; // server_id who performed it
  action: string; // "void" | "comp" | "pay" | "send" | "unpaid-close" | "void-check"
  target: string; // item_id or check_id
  reason_id?: string;
  timestamp: number;
  detail?: string;
}

/** Owner-managed app settings (§3 "everything configurable is data"). */
export interface Settings {
  /** Q5 — default word for the physical spot ("table" / "place" / …). */
  spot_label: string;
  /** Q4 — currency display. */
  currency_symbol: string;
}
