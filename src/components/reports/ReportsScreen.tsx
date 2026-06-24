import { useEffect, useState, type ReactNode } from "react";
import { useStore } from "@/store/useStore";
import { api, type DailyReport, type ReportSummary } from "@/lib/api";
import { Field } from "@/components/setup/ui";
import { money, classNames, todayStr } from "@/lib/util";

type Tab = "daily" | "global";

export function ReportsScreen() {
  const [tab, setTab] = useState<Tab>("daily");
  const TABS: [Tab, string][] = [
    ["daily", "Aujourd'hui"],
    ["global", "Global"],
  ];

  return (
    <div className="flex h-full flex-col">
      <header className="flex items-center gap-4 px-6 pt-5 pb-3">
        <h1 className="text-2xl font-extrabold tracking-tight">Rapports</h1>
        <div className="flex gap-1 rounded-chip bg-panel-2 p-1 ring-1 ring-line">
          {TABS.map(([id, label]) => (
            <button
              key={id}
              onClick={() => setTab(id)}
              className={classNames(
                "rounded-md px-4 py-1.5 text-sm font-medium",
                tab === id ? "bg-panel text-text ring-1 ring-line" : "text-muted hover:text-text",
              )}
            >
              {label}
            </button>
          ))}
        </div>
      </header>
      <div className="scroll-area flex-1 px-6 pb-8">
        {tab === "daily" ? <DailyView /> : <GlobalView />}
      </div>
    </div>
  );
}

// ---------------- daily ----------------

function DailyView() {
  const cur = useStore((s) => s.settings.currency_symbol);
  const [date, setDate] = useState(todayStr());
  const [data, setData] = useState<DailyReport | null>(null);
  const [error, setError] = useState<string | null>(null);

  const load = (d: string) => {
    setError(null);
    api
      .dailyReport(d)
      .then(setData)
      .catch((e) => setError((e as Error).message));
  };
  useEffect(() => load(date), [date]);

  return (
    <div className="max-w-3xl">
      <div className="mb-5 flex items-end gap-3">
        <Field label="Jour">
          <input
            type="date"
            value={date}
            onChange={(e) => setDate(e.target.value || todayStr())}
            className="h-10 rounded-chip bg-panel-2 px-3 text-sm text-text outline-none ring-1 ring-line focus:ring-2 focus:ring-blue"
          />
        </Field>
        <button
          onClick={() => load(date)}
          className="press h-10 rounded-chip bg-panel px-4 text-sm font-semibold ring-1 ring-line hover:bg-panel-2"
        >
          Actualiser
        </button>
        {data && (
          <button
            onClick={() => window.print()}
            className="press ml-auto h-10 rounded-chip bg-blue px-5 text-sm font-bold text-white hover:opacity-90"
          >
            Clôturer la journée
          </button>
        )}
      </div>

      {error && <p className="text-base text-coral">Échec du chargement : {error}</p>}
      {!data && !error && <p className="text-base text-muted">Chargement…</p>}

      {data && (
        <>
          <div className="grid grid-cols-3 gap-3">
            <Stat label="Ventes" value={money(data.sales, cur)} accent="mint" />
            <Stat label="Notes payées" value={String(data.paidCount)} />
            <Stat
              label="Impayés"
              value={String(data.unpaidCount)}
              accent={data.unpaidCount ? "coral" : undefined}
            />
          </div>

          {/* the key part: unpaid checks + who is responsible */}
          <Panel title="Impayés — qui est responsable">
            {data.unpaid.length === 0 ? (
              <p className="text-base font-semibold text-mint">Aucun impayé aujourd'hui ✓</p>
            ) : (
              <div className="flex flex-col gap-2">
                {data.unpaid.map((u) => (
                  <div
                    key={u.ticket_number}
                    className="flex items-center justify-between rounded-chip bg-coral/10 px-4 py-3 ring-1 ring-coral/30"
                  >
                    <div>
                      <div className="text-base font-bold">
                        Ticket nº {u.ticket_number} · {u.zone}
                      </div>
                      <div className="text-sm text-muted">{u.reason}</div>
                    </div>
                    <div className="text-right">
                      <div className="text-lg font-extrabold tabular-nums text-coral">
                        {money(u.amount, cur)}
                      </div>
                      <div className="text-sm text-muted">
                        Responsable : <span className="font-bold text-text">{u.server}</span>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </Panel>

          <div className="mt-6 grid grid-cols-1 gap-6 lg:grid-cols-2">
            <Panel title="Ventes par serveur">
              <BarList rows={data.byServer} total={data.sales} cur={cur} />
            </Panel>
            <Panel title="Annulations & offerts">
              <VoidCompList rows={data.voidComp} cur={cur} />
            </Panel>
          </div>

          <ClosingSummary data={data} cur={cur} />
        </>
      )}
    </div>
  );
}

// Printable end-of-day summary (hidden on screen; shown only when printing).
function ClosingSummary({ data, cur }: { data: DailyReport; cur: string }) {
  const [y, m, d] = data.date.split("-");
  const dateFr = `${d}/${m}/${y}`;
  return (
    <div id="closing-summary" className="mx-auto max-w-md text-black">
      <div className="text-center">
        <div className="text-xl font-extrabold">CAISSE</div>
        <div className="text-sm">Clôture de journée — {dateFr}</div>
      </div>

      <Line />
      <Row k="Ventes" v={money(data.sales, cur)} bold />
      <Row k="Notes payées" v={String(data.paidCount)} />
      <Row k="Impayés" v={String(data.unpaidCount)} />

      <Line />
      <div className="mb-1 text-sm font-bold uppercase">Ventes par serveur</div>
      {data.byServer.length === 0 ? (
        <div className="text-sm">Aucune vente.</div>
      ) : (
        data.byServer.map((s) => <Row key={s.label} k={s.label} v={money(s.amount, cur)} />)
      )}

      <Line />
      <div className="mb-1 text-sm font-bold uppercase">Impayés à récupérer</div>
      {data.unpaid.length === 0 ? (
        <div className="text-sm">Aucun impayé. ✓</div>
      ) : (
        data.unpaid.map((u) => (
          <div key={u.ticket_number} className="mb-1 text-sm">
            <div className="flex justify-between">
              <span>
                Ticket nº {u.ticket_number} · {u.zone} — <b>{u.server}</b>
              </span>
              <span className="font-bold tabular-nums">{money(u.amount, cur)}</span>
            </div>
            <div className="text-xs">{u.reason}</div>
          </div>
        ))
      )}

      {data.voidComp.length > 0 && (
        <>
          <Line />
          <div className="mb-1 text-sm font-bold uppercase">Annulations & offerts</div>
          {data.voidComp.map((r, i) => (
            <Row
              key={i}
              k={`${r.state === "VOID" ? "Annulé" : "Offert"} · ${r.label ?? "—"} ×${r.count}`}
              v={money(r.amount, cur)}
            />
          ))}
        </>
      )}

      <Line />
      <div className="mt-6 text-sm">Signature : _______________________</div>
      <div className="mt-2 text-xs">Édité le {dateFr}</div>
    </div>
  );
}

function Line() {
  return <div className="my-2 border-t border-black/30" />;
}

function Row({ k, v, bold }: { k: string; v: string; bold?: boolean }) {
  return (
    <div className={classNames("flex justify-between text-sm", bold && "font-bold")}>
      <span>{k}</span>
      <span className="tabular-nums">{v}</span>
    </div>
  );
}

// ---------------- global (all-time) ----------------

function GlobalView() {
  const cur = useStore((s) => s.settings.currency_symbol);
  const [data, setData] = useState<ReportSummary | null>(null);
  const [error, setError] = useState<string | null>(null);

  const load = () => {
    setError(null);
    api
      .reports()
      .then(setData)
      .catch((e) => setError((e as Error).message));
  };
  useEffect(load, []);

  if (error) return <p className="text-base text-coral">Échec du chargement : {error}</p>;
  if (!data) return <p className="text-base text-muted">Chargement…</p>;

  return (
    <>
      <div className="grid grid-cols-2 gap-3 md:grid-cols-4">
        <Stat label="Ventes" value={money(data.totalSales, cur)} accent="mint" />
        <Stat label="Notes payées" value={String(data.paidChecks)} />
        <Stat
          label="Impayés"
          value={String(data.unpaidChecks)}
          accent={data.unpaidChecks ? "coral" : undefined}
        />
        <Stat label="Annulé / offert" value={String(data.voidComp.length)} />
      </div>
      <div className="mt-6 grid grid-cols-1 gap-6 lg:grid-cols-2">
        <Panel title="Ventes par zone">
          <BarList rows={data.salesByZone} total={data.totalSales} cur={cur} />
        </Panel>
        <Panel title="Ventes par serveur">
          <BarList rows={data.salesByServer} total={data.totalSales} cur={cur} />
        </Panel>
        <Panel title="Annulations & offerts par motif">
          <VoidCompList rows={data.voidComp} cur={cur} />
        </Panel>
      </div>
    </>
  );
}

// ---------------- shared ----------------

function Stat({ label, value, accent }: { label: string; value: string; accent?: string }) {
  const color = accent ? `var(--color-${accent})` : undefined;
  return (
    <div className="rounded-card bg-panel p-4 ring-1 ring-line">
      <div className="text-sm text-muted">{label}</div>
      <div className="mt-1 text-3xl font-extrabold tabular-nums" style={{ color }}>
        {value}
      </div>
    </div>
  );
}

function Panel({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section className="mt-6 rounded-card bg-panel p-5 ring-1 ring-line">
      <h2 className="mb-3 text-base font-bold">{title}</h2>
      {children}
    </section>
  );
}

function VoidCompList({
  rows,
  cur,
}: {
  rows: { state: string; label: string | null; count: number; amount: number }[];
  cur: string;
}) {
  if (rows.length === 0) return <p className="text-base text-muted">Aucune annulation ni offert.</p>;
  return (
    <ul className="flex flex-col gap-2.5">
      {rows.map((r, i) => (
        <li key={i} className="flex items-center justify-between text-base">
          <span className="flex items-center gap-2">
            <span
              className={classNames(
                "rounded-full px-2.5 py-0.5 text-xs font-bold",
                r.state === "VOID" ? "bg-coral/15 text-coral" : "bg-purple/15 text-purple",
              )}
            >
              {r.state === "VOID" ? "Annulé" : "Offert"}
            </span>
            {r.label ?? "—"}
            <span className="text-muted">×{r.count}</span>
          </span>
          <span className="tabular-nums text-muted">{money(r.amount, cur)}</span>
        </li>
      ))}
    </ul>
  );
}

function BarList({
  rows,
  total,
  cur,
}: {
  rows: { label: string; amount: number }[];
  total: number;
  cur: string;
}) {
  if (rows.length === 0) return <p className="text-base text-muted">Aucune vente.</p>;
  const max = Math.max(...rows.map((r) => r.amount), 1);
  return (
    <ul className="flex flex-col gap-3">
      {rows.map((r) => (
        <li key={r.label}>
          <div className="flex items-center justify-between text-base">
            <span className="font-medium">{r.label}</span>
            <span className="tabular-nums text-muted">
              {money(r.amount, cur)}
              {total > 0 && <span className="ml-2 text-sm">{Math.round((r.amount / total) * 100)}%</span>}
            </span>
          </div>
          <div className="mt-1 h-2 overflow-hidden rounded-full bg-panel-2">
            <div className="h-full rounded-full bg-blue" style={{ width: `${(r.amount / max) * 100}%` }} />
          </div>
        </li>
      ))}
    </ul>
  );
}
