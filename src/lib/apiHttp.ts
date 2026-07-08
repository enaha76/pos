import type { Check, User } from "@/types/domain";
import type { Api, ConfigResponse, DailyReport, LoginResult, ReportSummary } from "@/lib/apiTypes";

const BASE = (import.meta.env.VITE_API_URL as string | undefined) ?? "http://localhost:8080";

// Current operator id, sent as X-User-Id so the backend can enforce roles.
let authUserId: string | null = null;
export function httpSetAuthUser(id: string | null): void {
  authUserId = id;
}

async function req<T>(path: string, options?: RequestInit): Promise<T> {
  const res = await fetch(`${BASE}${path}`, {
    ...options,
    headers: {
      "content-type": "application/json",
      ...(authUserId ? { "X-User-Id": authUserId } : {}),
      ...options?.headers,
    },
  });
  if (!res.ok) {
    let message = `${res.status} ${res.statusText}`;
    try {
      const body = await res.json();
      if (body?.error) message = body.error;
    } catch {
      /* non-JSON error body */
    }
    throw new Error(message);
  }
  return (await res.json()) as T;
}

const post = <T,>(path: string, body: unknown): Promise<T> =>
  req<T>(path, { method: "POST", body: JSON.stringify(body) });

/** WebSocket URL for the realtime change feed (derived from the API base). */
export const httpWsUrl = (): string => `${BASE.replace(/^http/, "ws")}/ws`;

export const httpApi: Api = {
  login: (pin) => post<LoginResult>("/api/login", { pin }),
  getConfig: () => req<ConfigResponse>("/api/config"),
  getOpenChecks: () => req<Check[]>("/api/checks/open"),

  createCheck: (b) => post<Check>("/api/checks", b),
  addItem: (b) => post<unknown>("/api/checks/add-item", b),
  setQty: (b) => post<unknown>("/api/items/set-qty", b),
  send: (check_id) => post<Check>("/api/checks/send", { check_id }),
  voidItem: (b) => post<unknown>("/api/items/void", b),
  compItem: (b) => post<unknown>("/api/items/comp", b),
  pay: (b) => post<Check>("/api/checks/pay", b),
  closeUnpaid: (b) => post<Check>("/api/checks/close-unpaid", b),
  setCheckServer: (b) => post<Check>("/api/checks/set-server", b),
  setCheckTable: (b) => post<Check>("/api/checks/set-table", b),

  reports: () => req<ReportSummary>("/api/reports/summary"),
  dailyReport: (date) => req<DailyReport>(`/api/reports/daily${date ? `?date=${date}` : ""}`),

  updateSettings: (patch) => post("/api/settings", patch),
  upsertZone: (zone) => post("/api/zones", zone),
  upsertCategory: (category) => post("/api/categories", category),
  upsertProduct: (product) => post("/api/products", product),
  upsertServer: (server) => post("/api/servers", server),
  upsertShift: (shift) => post("/api/shifts", shift),
  toggleAssignment: (b) => post<{ assigned: boolean }>("/api/assignments/toggle", b),

  listUsers: () => req<User[]>("/api/users"),
  createUser: (b) => post<User>("/api/users", b),
  updateUser: (b) => post<User>("/api/users/update", b),
  setUserPin: (b) => post<{ ok: boolean }>("/api/users/set-pin", b),
};
