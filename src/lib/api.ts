import { httpApi, httpSetAuthUser, httpWsUrl } from "@/lib/apiHttp";
import { sqliteApi } from "@/lib/apiSqlite";
import type { Api } from "@/lib/apiTypes";

/** True when running inside the Tauri desktop shell (embedded SQLite). */
export const isDesktop =
  typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

/** Data layer: embedded SQLite on desktop, HTTP backend in the browser. */
export const api: Api = isDesktop ? sqliteApi : httpApi;

/** Operator id for backend role checks (no-op on desktop — local SQLite). */
export const setAuthUser = isDesktop ? (_id: string | null) => {} : httpSetAuthUser;

/** Realtime WebSocket only exists with the HTTP backend. */
export const realtimeEnabled = !isDesktop;
export const wsUrl = httpWsUrl;

export type { ConfigResponse, ReportSummary, DailyReport } from "@/lib/apiTypes";
