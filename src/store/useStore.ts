import { create } from "zustand";
import { api, setAuthUser } from "@/lib/api";
import { id, inShift, todayStr } from "@/lib/util";
import type {
  Category,
  Check,
  Modifier,
  Product,
  ReasonCode,
  Server,
  Settings,
  Shift,
  ShiftAssignment,
  TableSpot,
  Zone,
} from "@/types/domain";

export type Route = "cashier" | "zones" | "products" | "reports" | "settings";
export type Role = "cashier" | "admin";
type Status = "loading" | "ready" | "error";

interface Session {
  user_id: string;
  name: string;
  role: Role;
  shift_id: string;
}

interface Draft {
  zone_id?: string;
  table_id?: string;
  table_label?: string;
  server_id?: string;
}

interface AppState {
  // ---- config (loaded from the API) ----
  settings: Settings;
  zones: Zone[];
  tableSpots: TableSpot[];
  servers: Server[];
  shifts: Shift[];
  categories: Category[];
  products: Product[];
  modifiers: Modifier[];
  reasonCodes: ReasonCode[];
  shiftAssignments: ShiftAssignment[];

  // ---- operational (open checks, from the API) ----
  checks: Check[];

  // ---- UI / session (local) ----
  session: Session | null;
  draft: Draft;
  activeCheckId: string | null;
  selectedItemId: string | null;
  route: Route;
  status: Status;
  error?: string;

  // ---- lifecycle ----
  bootstrap: () => Promise<void>;
  refreshChecks: () => Promise<void>;
  refreshConfig: () => Promise<void>;

  // ---- session ----
  login: (pin: string) => Promise<boolean>;
  logout: () => void;
  setRoute: (route: Route) => void;

  // ---- local UI selection ----
  selectZone: (zone_id: string) => void;
  setTable: (table_id?: string, table_label?: string) => Promise<void>;
  setOrderServer: (server_id: string) => Promise<void>;
  selectItem: (item_id: string | null) => void;
  openCheck: (check_id: string) => void;
  newCheck: () => void;

  // ---- cashier flow (API) ----
  addProduct: (product_id: string, modifier_ids?: string[]) => Promise<void>;
  incLine: (item_id: string, delta: number) => Promise<void>;
  send: () => Promise<void>;
  voidLine: (item_id: string, reason_id: string) => Promise<void>;
  compLine: (item_id: string, reason_id: string) => Promise<void>;
  pay: (method: string) => Promise<void>;
  closeUnpaid: (reason_id: string) => Promise<void>;

  // ---- config writes (optimistic local + API) ----
  updateSettings: (patch: Partial<Settings>) => Promise<void>;
  upsertZone: (zone: Zone) => Promise<void>;
  retireZone: (zone_id: string) => Promise<void>;
  upsertCategory: (category: Category) => Promise<void>;
  upsertProduct: (product: Product) => Promise<void>;
  retireProduct: (product_id: string) => Promise<void>;
  toggleAssignment: (
    server_id: string,
    zone_id: string,
    shift_id: string,
    date: string,
  ) => Promise<void>;
}

function resolveShift(shifts: Shift[]): Shift | undefined {
  const now = new Date();
  return shifts.find((s) => inShift(s.start_time, s.end_time, now)) ?? shifts[0];
}

/** Servers rostered to a zone for a shift on a date (§6.2 picker filter). */
function rosteredServers(
  assignments: ShiftAssignment[],
  zone_id: string | undefined,
  shift_id: string | undefined,
  date: string,
): string[] {
  if (!zone_id || !shift_id) return [];
  return assignments
    .filter((a) => a.zone_id === zone_id && a.shift_id === shift_id && a.date === date)
    .map((a) => a.server_id);
}

export const useStore = create<AppState>()((set, get) => ({
  settings: { spot_label: "Table", currency_symbol: "MRU" },
  zones: [],
  tableSpots: [],
  servers: [],
  shifts: [],
  categories: [],
  products: [],
  modifiers: [],
  reasonCodes: [],
  shiftAssignments: [],
  checks: [],

  session: null,
  draft: {},
  activeCheckId: null,
  selectedItemId: null,
  route: "cashier",
  status: "loading",

  bootstrap: async () => {
    set({ status: "loading", error: undefined });
    try {
      const [config, checks] = await Promise.all([api.getConfig(), api.getOpenChecks()]);
      set({ ...config, checks, status: "ready" });
    } catch (e) {
      set({ status: "error", error: (e as Error).message });
    }
  },

  refreshChecks: async () => {
    set({ checks: await api.getOpenChecks() });
  },

  refreshConfig: async () => {
    set({ ...(await api.getConfig()) });
  },

  login: async (pin) => {
    try {
      const user = await api.login(pin); // validated server-side (bcrypt)
      const shift = resolveShift(get().shifts);
      setAuthUser(user.user_id); // sent as X-User-Id for backend role checks
      set({
        session: { user_id: user.user_id, name: user.name, role: user.role, shift_id: shift?.shift_id ?? "" },
        draft: {},
        route: "cashier",
      });
      return true;
    } catch {
      return false;
    }
  },

  logout: () => {
    setAuthUser(null);
    set({ session: null, draft: {}, activeCheckId: null, selectedItemId: null });
  },

  setRoute: (route) => set({ route }),

  selectZone: (zone_id) => {
    const { draft, session, shiftAssignments } = get();
    // §6.1 — resolve the server rostered to this zone for the current shift.
    // (The operator is a cashier/admin, not a server, so there's no "self" default.)
    const rostered = rosteredServers(shiftAssignments, zone_id, session?.shift_id, todayStr());
    const prev = draft.server_id;
    const server_id =
      prev && rostered.includes(prev) ? prev : rostered[0]; // first rostered, or undefined
    set({
      draft: { zone_id, table_id: undefined, table_label: undefined, server_id },
      activeCheckId: null,
      selectedItemId: null,
    });
  },

  setTable: async (table_id, table_label) => {
    set((s) => ({ draft: { ...s.draft, table_id, table_label } }));
    const { activeCheckId } = get();
    if (activeCheckId) {
      await api.setCheckTable({ check_id: activeCheckId, table_id, table_label });
      await get().refreshChecks();
    }
  },

  setOrderServer: async (server_id) => {
    set((s) => ({ draft: { ...s.draft, server_id } }));
    const { activeCheckId } = get();
    if (activeCheckId) {
      await api.setCheckServer({ check_id: activeCheckId, server_id });
      await get().refreshChecks();
    }
  },

  selectItem: (item_id) => set({ selectedItemId: item_id }),

  openCheck: (check_id) => {
    const c = get().checks.find((x) => x.check_id === check_id);
    if (!c) return;
    set({
      activeCheckId: check_id,
      draft: {
        zone_id: c.zone_id,
        table_id: c.table_id,
        table_label: c.table_label,
        server_id: c.server_id,
      },
      selectedItemId: null,
    });
  },

  newCheck: () =>
    set((s) => ({
      activeCheckId: null,
      selectedItemId: null,
      draft: { ...s.draft, table_id: undefined, table_label: undefined },
    })),

  addProduct: async (product_id, modifier_ids) => {
    const { session, draft } = get();
    if (!session || !draft.zone_id || !draft.server_id) return; // a check needs a server
    try {
      let checkId = get().activeCheckId;
      if (!checkId) {
        const check = await api.createCheck({
          zone_id: draft.zone_id,
          server_id: draft.server_id,
          table_id: draft.table_id,
          table_label: draft.table_label,
        });
        checkId = check.check_id;
        set({ activeCheckId: checkId });
      }
      await api.addItem({ check_id: checkId, product_id, qty: 1, modifier_ids });
      await get().refreshChecks();
    } catch (e) {
      set({ error: (e as Error).message });
    }
  },

  incLine: async (item_id, delta) => {
    const { activeCheckId, checks } = get();
    if (!activeCheckId) return;
    const item = checks.find((c) => c.check_id === activeCheckId)?.items.find((i) => i.item_id === item_id);
    if (!item) return;
    await api.setQty({ item_id, qty: item.qty + delta });
    await get().refreshChecks();
  },

  send: async () => {
    const { activeCheckId } = get();
    if (!activeCheckId) return;
    await api.send(activeCheckId);
    await get().refreshChecks();
  },

  voidLine: async (item_id, reason_id) => {
    await api.voidItem({ item_id, reason_id });
    set({ selectedItemId: null });
    await get().refreshChecks();
  },

  compLine: async (item_id, reason_id) => {
    await api.compItem({ item_id, reason_id });
    set({ selectedItemId: null });
    await get().refreshChecks();
  },

  pay: async (method) => {
    const { activeCheckId } = get();
    if (!activeCheckId) return;
    await api.pay({ check_id: activeCheckId, method });
    set({ activeCheckId: null, selectedItemId: null });
    await get().refreshChecks();
  },

  closeUnpaid: async (reason_id) => {
    const { activeCheckId } = get();
    if (!activeCheckId) return;
    await api.closeUnpaid({ check_id: activeCheckId, reason_id });
    set({ activeCheckId: null, selectedItemId: null });
    await get().refreshChecks();
  },

  // ---- config writes: optimistic local update + persist (keeps typing smooth) ----
  updateSettings: async (patch) => {
    set((s) => ({ settings: { ...s.settings, ...patch } }));
    try {
      await api.updateSettings(patch);
    } catch (e) {
      set({ error: (e as Error).message });
    }
  },

  upsertZone: async (zone) => {
    set((s) => ({
      zones: s.zones.some((z) => z.zone_id === zone.zone_id)
        ? s.zones.map((z) => (z.zone_id === zone.zone_id ? zone : z))
        : [...s.zones, zone],
    }));
    try {
      await api.upsertZone(zone);
    } catch (e) {
      set({ error: (e as Error).message });
    }
  },

  retireZone: async (zone_id) => {
    set((s) => ({ zones: s.zones.map((z) => (z.zone_id === zone_id ? { ...z, active: false } : z)) }));
    const zone = get().zones.find((z) => z.zone_id === zone_id);
    if (zone) await api.upsertZone(zone).catch((e) => set({ error: (e as Error).message }));
  },

  upsertCategory: async (category) => {
    set((s) => ({
      categories: s.categories.some((c) => c.category_id === category.category_id)
        ? s.categories.map((c) => (c.category_id === category.category_id ? category : c))
        : [...s.categories, category],
    }));
    try {
      await api.upsertCategory(category);
    } catch (e) {
      set({ error: (e as Error).message });
    }
  },

  upsertProduct: async (product) => {
    set((s) => ({
      products: s.products.some((p) => p.product_id === product.product_id)
        ? s.products.map((p) => (p.product_id === product.product_id ? product : p))
        : [...s.products, product],
    }));
    try {
      await api.upsertProduct(product);
    } catch (e) {
      set({ error: (e as Error).message });
    }
  },

  retireProduct: async (product_id) => {
    set((s) => ({
      products: s.products.map((p) => (p.product_id === product_id ? { ...p, active: false } : p)),
    }));
    const product = get().products.find((p) => p.product_id === product_id);
    if (product) await api.upsertProduct(product).catch((e) => set({ error: (e as Error).message }));
  },

  toggleAssignment: async (server_id, zone_id, shift_id, date) => {
    set((s) => {
      const existing = s.shiftAssignments.find(
        (a) =>
          a.server_id === server_id &&
          a.zone_id === zone_id &&
          a.shift_id === shift_id &&
          a.date === date,
      );
      return {
        shiftAssignments: existing
          ? s.shiftAssignments.filter((a) => a !== existing)
          : [...s.shiftAssignments, { assignment_id: id("asn"), server_id, zone_id, shift_id, date }],
      };
    });
    try {
      await api.toggleAssignment({ server_id, zone_id, shift_id, date });
    } catch (e) {
      set({ error: (e as Error).message });
    }
  },
}));
