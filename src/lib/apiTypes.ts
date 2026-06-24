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
  User,
  Zone,
} from "@/types/domain";

export interface ConfigResponse {
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
}

export interface ReportSummary {
  totalSales: number;
  paidChecks: number;
  unpaidChecks: number;
  salesByZone: { label: string; amount: number }[];
  salesByServer: { label: string; amount: number }[];
  voidComp: { state: string; label: string | null; count: number; amount: number }[];
}

export interface DailyReport {
  date: string;
  sales: number;
  paidCount: number;
  unpaidCount: number;
  unpaid: { ticket_number: number; server: string; zone: string; reason: string; amount: number }[];
  byServer: { label: string; amount: number }[];
  voidComp: { state: string; label: string | null; count: number; amount: number }[];
}

export interface LoginResult {
  user_id: string;
  name: string;
  role: "cashier" | "admin";
}

/** Contract both the HTTP backend and the embedded SQLite layer implement. */
export interface Api {
  login(pin: string): Promise<LoginResult>;
  getConfig(): Promise<ConfigResponse>;
  getOpenChecks(): Promise<Check[]>;
  createCheck(b: {
    zone_id: string;
    server_id: string;
    table_id?: string;
    table_label?: string;
  }): Promise<Check>;
  addItem(b: {
    check_id: string;
    product_id: string;
    qty?: number;
    modifier_ids?: string[];
  }): Promise<unknown>;
  setQty(b: { item_id: string; qty: number }): Promise<unknown>;
  send(check_id: string): Promise<Check>;
  voidItem(b: { item_id: string; reason_id: string }): Promise<unknown>;
  compItem(b: { item_id: string; reason_id: string }): Promise<unknown>;
  pay(b: { check_id: string; method: string }): Promise<Check>;
  closeUnpaid(b: { check_id: string; reason_id: string }): Promise<Check>;
  setCheckServer(b: { check_id: string; server_id: string }): Promise<Check>;
  setCheckTable(b: { check_id: string; table_id?: string; table_label?: string }): Promise<Check>;
  reports(): Promise<ReportSummary>;
  dailyReport(date?: string): Promise<DailyReport>;
  updateSettings(patch: Partial<Settings>): Promise<Settings>;
  upsertZone(zone: Zone): Promise<Zone>;
  upsertCategory(category: Category): Promise<Category>;
  upsertProduct(product: Product): Promise<Product>;
  toggleAssignment(b: {
    server_id: string;
    zone_id: string;
    shift_id: string;
    date: string;
  }): Promise<{ assigned: boolean }>;
  listUsers(): Promise<User[]>;
  createUser(b: { name: string; role: string; pin: string }): Promise<User>;
  updateUser(b: { user_id: string; name: string; role: string; active: boolean }): Promise<User>;
  setUserPin(b: { user_id: string; pin: string }): Promise<{ ok: boolean }>;
}
