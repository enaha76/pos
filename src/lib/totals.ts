import type { Check, OrderItem } from "@/types/domain";

export interface Totals {
  subtotal: number; // chargeable lines (HELD + SENT)
  comps: number; // COMP lines — food used, off the bill
  voids: number; // VOID lines — no cost (informational)
  total: number; // amount due
  heldQty: number;
  sentQty: number;
  canSend: boolean; // §6.1.5 — held items exist
  canPay: boolean; // §6.3 — chargeable items exist and nothing still held
}

const amt = (it: OrderItem) => it.qty * it.unit_price;

export function checkTotals(check: Check | undefined | null): Totals {
  const t: Totals = {
    subtotal: 0,
    comps: 0,
    voids: 0,
    total: 0,
    heldQty: 0,
    sentQty: 0,
    canSend: false,
    canPay: false,
  };
  if (!check) return t;

  for (const it of check.items) {
    switch (it.state) {
      case "HELD":
        t.subtotal += amt(it);
        t.heldQty += it.qty;
        break;
      case "SENT":
        t.subtotal += amt(it);
        t.sentQty += it.qty;
        break;
      case "COMP":
        t.comps += amt(it);
        break;
      case "VOID":
        t.voids += amt(it);
        break;
    }
  }
  t.total = t.subtotal;
  t.canSend = t.heldQty > 0;
  t.canPay = t.sentQty > 0 && t.heldQty === 0;
  return t;
}
