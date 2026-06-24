import Database from "@tauri-apps/plugin-sql";
import { id, todayStr } from "@/lib/util";
import type { Check, OrderItem, Payment, Product, Settings, User, Zone } from "@/types/domain";
import type { Api, ConfigResponse, DailyReport, ReportSummary } from "@/lib/apiTypes";

// ---- connection (opened once; seeds PINs to SHA-256 on first run) ----
let _db: Promise<Database> | null = null;
function db(): Promise<Database> {
  return (_db ??= open());
}
async function open(): Promise<Database> {
  const d = await Database.load("sqlite:caisse.db");
  await hashSeedPins(d);
  return d;
}

async function sha256(text: string): Promise<string> {
  const buf = await crypto.subtle.digest("SHA-256", new TextEncoder().encode(text));
  return [...new Uint8Array(buf)].map((b) => b.toString(16).padStart(2, "0")).join("");
}

// Seed PINs are stored in plaintext; hash any that aren't yet 64-hex SHA-256.
async function hashSeedPins(d: Database): Promise<void> {
  const rows = await d.select<{ user_id: string; pin_hash: string }[]>(
    "select user_id, pin_hash from users",
  );
  for (const r of rows) {
    if (!/^[0-9a-f]{64}$/.test(r.pin_hash)) {
      await d.execute("update users set pin_hash = $2 where user_id = $1", [
        r.user_id,
        await sha256(r.pin_hash),
      ]);
    }
  }
}

const nowIso = () => new Date().toISOString();
const b2i = (v: boolean) => (v ? 1 : 0);

async function audit(
  d: Database,
  actor: string | null,
  action: string,
  target: string,
  reason: string | null,
  detail: string | null,
): Promise<void> {
  await d.execute(
    "insert into audit_log (log_id, actor_id, action, target, reason_id, detail, timestamp) \
     values ($1,$2,$3,$4,$5,$6,$7)",
    [id("log"), actor, action, target, reason, detail, nowIso()],
  );
}

async function loadCheck(d: Database, checkId: string): Promise<Check> {
  const checks = await d.select<Check[]>("select * from checks where check_id = $1", [checkId]);
  const items = await d.select<OrderItem[]>(
    "select * from order_items where check_id = $1 order by created_at",
    [checkId],
  );
  const payments = await d.select<Payment[]>("select * from payments where check_id = $1", [checkId]);
  return { ...checks[0], items, payments };
}

export const sqliteApi: Api = {
  login: async (pin) => {
    const d = await db();
    const rows = await d.select<{ user_id: string; name: string; role: "cashier" | "admin" }[]>(
      "select user_id, name, role from users where active = 1 and pin_hash = $1 limit 1",
      [await sha256(pin)],
    );
    if (!rows.length) throw new Error("invalid PIN");
    return rows[0];
  },

  getConfig: async () => {
    const d = await db();
    const [settings] = await d.select<Settings[]>(
      "select spot_label, currency_symbol from settings where id = 1",
    );
    const zones = await d.select<Zone[]>("select * from zones order by display_order");
    const tableSpots = await d.select("select * from table_spots");
    const servers = await d.select("select * from servers");
    const shifts = await d.select("select * from shifts");
    const categories = await d.select("select * from categories order by display_order");
    const products = await d.select("select * from products");
    const modifiers = await d.select("select * from modifiers");
    const reasonCodes = await d.select("select * from reason_codes");
    const shiftAssignments = await d.select("select * from shift_assignments");
    const onActive = <T extends { active?: number | boolean }>(rows: T[]) =>
      rows.map((r) => ({ ...r, active: !!r.active }));
    return {
      settings,
      zones: onActive(zones as Zone[]),
      tableSpots: onActive(tableSpots as { active: number }[]),
      servers: onActive(servers as { active: number }[]),
      shifts,
      categories,
      products: onActive(products as Product[]),
      modifiers,
      reasonCodes: onActive(reasonCodes as { active: number }[]),
      shiftAssignments,
    } as ConfigResponse;
  },

  getOpenChecks: async () => {
    const d = await db();
    const checks = await d.select<Check[]>(
      "select * from checks where status in ('OPEN','IN_PROGRESS') order by ticket_number",
    );
    const out: Check[] = [];
    for (const c of checks) out.push(await loadCheck(d, c.check_id));
    return out;
  },

  createCheck: async (b) => {
    const d = await db();
    const [{ next }] = await d.select<{ next: number }[]>(
      "select coalesce(max(ticket_number),0) + 1 as next from checks",
    );
    const checkId = id("chk");
    await d.execute(
      "insert into checks (check_id, ticket_number, zone_id, server_id, table_id, table_label, status, opened_at) \
       values ($1,$2,$3,$4,$5,$6,'OPEN',$7)",
      [checkId, next, b.zone_id, b.server_id, b.table_id ?? null, b.table_label ?? null, nowIso()],
    );
    return loadCheck(d, checkId);
  },

  addItem: async (b) => {
    const d = await db();
    const [product] = await d.select<Product[]>(
      "select * from products where product_id = $1 and active = 1",
      [b.product_id],
    );
    if (!product) throw new Error("unknown product");
    const [check] = await d.select<Check[]>("select * from checks where check_id = $1", [b.check_id]);
    if (!check) throw new Error("not found");

    const qty = Math.max(1, b.qty ?? 1);
    let lineName = product.name;
    let linePrice = product.price;
    const ids = b.modifier_ids ?? [];
    if (ids.length) {
      const ph = ids.map((_, i) => `$${i + 2}`).join(",");
      const mods = await d.select<{ name: string; price_delta: number }[]>(
        `select name, price_delta from modifiers where product_id = $1 and modifier_id in (${ph}) order by name`,
        [product.product_id, ...ids],
      );
      if (mods.length) {
        lineName = `${product.name} · ${mods.map((m) => m.name).join(", ")}`;
        linePrice = product.price + mods.reduce((s, m) => s + m.price_delta, 0);
      }
    }

    const existing = await d.select<{ item_id: string }[]>(
      "select item_id from order_items where check_id = $1 and name = $2 and state = 'HELD' order by created_at limit 1",
      [b.check_id, lineName],
    );
    if (existing.length) {
      await d.execute("update order_items set qty = qty + $2 where item_id = $1", [
        existing[0].item_id,
        qty,
      ]);
    } else {
      await d.execute(
        "insert into order_items (item_id, check_id, server_id, product_id, name, qty, unit_price, state, created_at) \
         values ($1,$2,$3,$4,$5,$6,$7,'HELD',$8)",
        [id("itm"), b.check_id, check.server_id, product.product_id, lineName, qty, linePrice, nowIso()],
      );
    }
    return { ok: true };
  },

  setQty: async (b) => {
    const d = await db();
    if (b.qty <= 0) {
      await d.execute("delete from order_items where item_id = $1 and state = 'HELD'", [b.item_id]);
    } else {
      await d.execute("update order_items set qty = $2 where item_id = $1 and state = 'HELD'", [
        b.item_id,
        b.qty,
      ]);
    }
    return { ok: true };
  },

  send: async (check_id) => {
    const d = await db();
    const [check] = await d.select<Check[]>("select * from checks where check_id = $1", [check_id]);
    const res = await d.execute(
      "update order_items set state = 'SENT' where check_id = $1 and state = 'HELD'",
      [check_id],
    );
    if (res.rowsAffected > 0) {
      await d.execute("update checks set status = 'IN_PROGRESS' where check_id = $1", [check_id]);
      await audit(d, check?.server_id ?? null, "send", check_id, null, `${res.rowsAffected} line(s)`);
    }
    return loadCheck(d, check_id);
  },

  voidItem: async (b) => {
    const d = await db();
    const [item] = await d.select<OrderItem[]>("select * from order_items where item_id = $1", [
      b.item_id,
    ]);
    await d.execute("update order_items set state = 'VOID', reason_id = $2 where item_id = $1", [
      b.item_id,
      b.reason_id,
    ]);
    await audit(d, item?.server_id ?? null, "void", b.item_id, b.reason_id, null);
    return { ok: true };
  },

  compItem: async (b) => {
    const d = await db();
    const [item] = await d.select<OrderItem[]>("select * from order_items where item_id = $1", [
      b.item_id,
    ]);
    await d.execute("update order_items set state = 'COMP', reason_id = $2 where item_id = $1", [
      b.item_id,
      b.reason_id,
    ]);
    await audit(d, item?.server_id ?? null, "comp", b.item_id, b.reason_id, null);
    return { ok: true };
  },

  pay: async (b) => {
    const d = await db();
    const [check] = await d.select<Check[]>("select * from checks where check_id = $1", [b.check_id]);
    const [{ due }] = await d.select<{ due: number }[]>(
      "select coalesce(sum(qty * unit_price), 0) as due from order_items where check_id = $1 and state in ('HELD','SENT')",
      [b.check_id],
    );
    if (due <= 0) throw new Error("nothing to pay");
    await d.execute(
      "insert into payments (payment_id, check_id, method, amount, paid_at) values ($1,$2,$3,$4,$5)",
      [id("pay"), b.check_id, b.method, due, nowIso()],
    );
    await d.execute("update checks set status = 'CLOSED_PAID', closed_at = $2 where check_id = $1", [
      b.check_id,
      nowIso(),
    ]);
    await audit(d, check?.server_id ?? null, "pay", b.check_id, null, `${b.method} ${due}`);
    return loadCheck(d, b.check_id);
  },

  closeUnpaid: async (b) => {
    const d = await db();
    const [check] = await d.select<Check[]>("select * from checks where check_id = $1", [b.check_id]);
    await d.execute(
      "update checks set status = 'CLOSED_UNPAID', reason_id = $2, closed_at = $3 where check_id = $1",
      [b.check_id, b.reason_id, nowIso()],
    );
    await audit(d, check?.server_id ?? null, "unpaid-close", b.check_id, b.reason_id, null);
    return loadCheck(d, b.check_id);
  },

  setCheckServer: async (b) => {
    const d = await db();
    await d.execute("update checks set server_id = $2 where check_id = $1", [b.check_id, b.server_id]);
    await d.execute("update order_items set server_id = $2 where check_id = $1", [
      b.check_id,
      b.server_id,
    ]);
    return loadCheck(d, b.check_id);
  },

  setCheckTable: async (b) => {
    const d = await db();
    await d.execute("update checks set table_id = $2, table_label = $3 where check_id = $1", [
      b.check_id,
      b.table_id ?? null,
      b.table_label ?? null,
    ]);
    return loadCheck(d, b.check_id);
  },

  reports: async () => {
    const d = await db();
    const scalar = async (sql: string, p: unknown[] = []) =>
      (await d.select<{ v: number }[]>(sql, p))[0]?.v ?? 0;
    return {
      totalSales: await scalar(
        "select coalesce(sum(p.amount),0) as v from payments p join checks c on c.check_id = p.check_id where c.status = 'CLOSED_PAID'",
      ),
      paidChecks: await scalar("select count(*) as v from checks where status = 'CLOSED_PAID'"),
      unpaidChecks: await scalar("select count(*) as v from checks where status = 'CLOSED_UNPAID'"),
      salesByZone: await d.select(
        "select z.name as label, coalesce(sum(p.amount),0) as amount from checks c join payments p on p.check_id = c.check_id join zones z on z.zone_id = c.zone_id where c.status = 'CLOSED_PAID' group by z.name order by amount desc",
      ),
      salesByServer: await d.select(
        "select s.name as label, coalesce(sum(p.amount),0) as amount from checks c join payments p on p.check_id = c.check_id join servers s on s.server_id = c.server_id where c.status = 'CLOSED_PAID' group by s.name order by amount desc",
      ),
      voidComp: await d.select(
        "select oi.state as state, rc.label as label, sum(oi.qty) as count, sum(oi.qty * oi.unit_price) as amount from order_items oi left join reason_codes rc on rc.reason_id = oi.reason_id where oi.state in ('VOID','COMP') group by oi.state, rc.label",
      ),
    } as ReportSummary;
  },

  dailyReport: async (date) => {
    const d = await db();
    const day = date ?? todayStr();
    const scalar = async (sql: string, p: unknown[]) =>
      (await d.select<{ v: number }[]>(sql, p))[0]?.v ?? 0;
    const unpaid = await d.select<DailyReport["unpaid"]>(
      "select c.ticket_number as ticket_number, s.name as server, z.name as zone, \
         coalesce(rc.label,'—') as reason, \
         coalesce((select sum(oi.qty*oi.unit_price) from order_items oi where oi.check_id = c.check_id and oi.state in ('HELD','SENT')),0) as amount \
       from checks c join servers s on s.server_id = c.server_id join zones z on z.zone_id = c.zone_id \
       left join reason_codes rc on rc.reason_id = c.reason_id \
       where c.status = 'CLOSED_UNPAID' and date(c.opened_at) = $1 order by c.ticket_number",
      [day],
    );
    return {
      date: day,
      sales: await scalar(
        "select coalesce(sum(p.amount),0) as v from payments p join checks c on c.check_id = p.check_id where c.status = 'CLOSED_PAID' and date(c.opened_at) = $1",
        [day],
      ),
      paidCount: await scalar(
        "select count(*) as v from checks where status = 'CLOSED_PAID' and date(opened_at) = $1",
        [day],
      ),
      unpaidCount: unpaid.length,
      unpaid,
      byServer: await d.select(
        "select s.name as label, coalesce(sum(p.amount),0) as amount from checks c join payments p on p.check_id = c.check_id join servers s on s.server_id = c.server_id where c.status = 'CLOSED_PAID' and date(c.opened_at) = $1 group by s.name order by amount desc",
        [day],
      ),
      voidComp: await d.select(
        "select oi.state as state, rc.label as label, sum(oi.qty) as count, sum(oi.qty*oi.unit_price) as amount from order_items oi join checks c on c.check_id = oi.check_id left join reason_codes rc on rc.reason_id = oi.reason_id where oi.state in ('VOID','COMP') and date(c.opened_at) = $1 group by oi.state, rc.label",
        [day],
      ),
    } as DailyReport;
  },

  updateSettings: async (patch) => {
    const d = await db();
    await d.execute(
      "update settings set spot_label = coalesce($1, spot_label), currency_symbol = coalesce($2, currency_symbol) where id = 1",
      [patch.spot_label ?? null, patch.currency_symbol ?? null],
    );
    const [s] = await d.select<Settings[]>("select spot_label, currency_symbol from settings where id = 1");
    return s;
  },

  upsertZone: async (zone) => {
    const d = await db();
    await d.execute(
      "insert into zones (zone_id, name, display_order, table_mode, spot_label, active) values ($1,$2,$3,$4,$5,$6) \
       on conflict(zone_id) do update set name=excluded.name, display_order=excluded.display_order, table_mode=excluded.table_mode, spot_label=excluded.spot_label, active=excluded.active",
      [zone.zone_id, zone.name, zone.display_order, zone.table_mode, zone.spot_label ?? null, b2i(zone.active)],
    );
    return zone;
  },

  upsertCategory: async (category) => {
    const d = await db();
    await d.execute(
      "insert into categories (category_id, name, color, display_order) values ($1,$2,$3,$4) \
       on conflict(category_id) do update set name=excluded.name, color=excluded.color, display_order=excluded.display_order",
      [category.category_id, category.name, category.color, category.display_order],
    );
    return category;
  },

  upsertProduct: async (product) => {
    const d = await db();
    await d.execute(
      "insert into products (product_id, name, category_id, price, active) values ($1,$2,$3,$4,$5) \
       on conflict(product_id) do update set name=excluded.name, category_id=excluded.category_id, price=excluded.price, active=excluded.active",
      [product.product_id, product.name, product.category_id, product.price, b2i(product.active)],
    );
    return product;
  },

  toggleAssignment: async (b) => {
    const d = await db();
    const existing = await d.select<{ assignment_id: string }[]>(
      "select assignment_id from shift_assignments where server_id = $1 and zone_id = $2 and shift_id = $3 and date = $4",
      [b.server_id, b.zone_id, b.shift_id, b.date],
    );
    if (existing.length) {
      await d.execute("delete from shift_assignments where assignment_id = $1", [existing[0].assignment_id]);
      return { assigned: false };
    }
    await d.execute(
      "insert into shift_assignments (assignment_id, server_id, zone_id, shift_id, date) values ($1,$2,$3,$4,$5)",
      [id("asn"), b.server_id, b.zone_id, b.shift_id, b.date],
    );
    return { assigned: true };
  },

  listUsers: async () => {
    const d = await db();
    const rows = await d.select<{ user_id: string; name: string; role: "cashier" | "admin"; active: number }[]>(
      "select user_id, name, role, active from users order by name",
    );
    return rows.map((u) => ({ ...u, active: !!u.active })) as User[];
  },

  createUser: async (b) => {
    const d = await db();
    if (!b.name.trim()) throw new Error("nom requis");
    if (b.role !== "cashier" && b.role !== "admin") throw new Error("rôle invalide");
    if (b.pin.length < 4) throw new Error("le code doit faire au moins 4 chiffres");
    const hash = await sha256(b.pin);
    const dup = await d.select<{ n: number }[]>(
      "select count(*) as n from users where pin_hash = $1 and active = 1",
      [hash],
    );
    if (dup[0].n > 0) throw new Error("ce code est déjà utilisé");
    const userId = id("usr");
    await d.execute(
      "insert into users (user_id, name, pin_hash, role, active) values ($1,$2,$3,$4,1)",
      [userId, b.name.trim(), hash, b.role],
    );
    return { user_id: userId, name: b.name.trim(), role: b.role as "cashier" | "admin", active: true };
  },

  updateUser: async (b) => {
    const d = await db();
    if (!b.name.trim()) throw new Error("nom requis");
    if (b.role !== "cashier" && b.role !== "admin") throw new Error("rôle invalide");
    if (b.role !== "admin" || !b.active) {
      const [{ n }] = await d.select<{ n: number }[]>(
        "select count(*) as n from users where role = 'admin' and active = 1 and user_id <> $1",
        [b.user_id],
      );
      if (n === 0) throw new Error("au moins un administrateur actif est requis");
    }
    await d.execute("update users set name = $2, role = $3, active = $4 where user_id = $1", [
      b.user_id,
      b.name.trim(),
      b.role,
      b2i(b.active),
    ]);
    return { user_id: b.user_id, name: b.name.trim(), role: b.role as "cashier" | "admin", active: b.active };
  },

  setUserPin: async (b) => {
    const d = await db();
    if (b.pin.length < 4) throw new Error("le code doit faire au moins 4 chiffres");
    const hash = await sha256(b.pin);
    const dup = await d.select<{ n: number }[]>(
      "select count(*) as n from users where pin_hash = $1 and active = 1 and user_id <> $2",
      [hash, b.user_id],
    );
    if (dup[0].n > 0) throw new Error("ce code est déjà utilisé");
    await d.execute("update users set pin_hash = $2 where user_id = $1", [b.user_id, hash]);
    return { ok: true };
  },
};
