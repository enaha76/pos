/** Small helpers shared across the app. */

/** Business name shown on printed documents (facture, end-of-day summary). */
export const BUSINESS_NAME = "Café Adalya";

let _counter = 0;
/** Monotonic-ish id; fine for a single-station client store. */
export function id(prefix = "id"): string {
  _counter += 1;
  return `${prefix}_${Date.now().toString(36)}${_counter.toString(36)}`;
}

// MRU (ouguiya) is used in whole units, with French number grouping.
const frInt = new Intl.NumberFormat("fr-FR", { maximumFractionDigits: 0 });

/** Money is stored as whole ouguiya. Format for display, e.g. "2 800 MRU". */
export function money(amount: number, symbol = "MRU"): string {
  const sign = amount < 0 ? "-" : "";
  return `${sign}${frInt.format(Math.abs(amount))} ${symbol}`;
}

export function classNames(...parts: Array<string | false | null | undefined>): string {
  return parts.filter(Boolean).join(" ");
}

/** Local date as YYYY-MM-DD, used to key shift assignments to a day. */
export function todayStr(d = new Date()): string {
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  return `${y}-${m}-${day}`;
}

/** "07:00"–"16:00" style range helpers for shift resolution. */
function minutesOf(t: string): number {
  const [h, m] = t.split(":").map(Number);
  return h * 60 + m;
}

/** Is `now` (Date) inside [start,end), tolerating an overnight wrap (e.g. 16:00–01:00)? */
export function inShift(start: string, end: string, now: Date): boolean {
  const cur = now.getHours() * 60 + now.getMinutes();
  const s = minutesOf(start);
  const e = minutesOf(end);
  return s <= e ? cur >= s && cur < e : cur >= s || cur < e;
}
