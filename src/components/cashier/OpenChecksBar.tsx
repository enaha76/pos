import { useStore } from "@/store/useStore";
import { checkTotals } from "@/lib/totals";
import { classNames, money } from "@/lib/util";
import type { Check } from "@/types/domain";

/**
 * Strip of all currently-open checks so the cashier can run several tables at
 * once and tap between them. "+ New" parks the current check and starts another.
 */
export function OpenChecksBar() {
  const checks = useStore((s) => s.checks);
  const activeCheckId = useStore((s) => s.activeCheckId);
  const openCheck = useStore((s) => s.openCheck);
  const newCheck = useStore((s) => s.newCheck);
  const zones = useStore((s) => s.zones);
  const tableSpots = useStore((s) => s.tableSpots);
  const servers = useStore((s) => s.servers);
  const settings = useStore((s) => s.settings);

  const open = checks.filter((c) => c.status === "OPEN" || c.status === "IN_PROGRESS");

  const placeOf = (c: Check) => {
    const zone = zones.find((z) => z.zone_id === c.zone_id);
    const spot = zone?.spot_label ?? settings.spot_label;
    const t = c.table_label ?? tableSpots.find((ts) => ts.table_id === c.table_id)?.label;
    return zone ? `${zone.name}${t ? ` · ${spot} ${t}` : ""}` : "—";
  };
  const serverOf = (id: string) => servers.find((s) => s.server_id === id)?.name ?? "";

  return (
    <div className="flex items-center gap-2 overflow-x-auto border-b border-line bg-panel px-4 py-3">
      <button
        onClick={newCheck}
        className={classNames(
          "press flex h-16 shrink-0 items-center gap-2 rounded-chip px-5 text-base font-bold ring-1",
          !activeCheckId ? "bg-blue text-white ring-transparent" : "bg-panel text-blue ring-blue hover:bg-panel-2",
        )}
      >
        + Nouvelle
      </button>

      <div className="mx-1 h-10 w-px shrink-0 bg-line" />

      {open.length === 0 ? (
        <span className="px-3 text-base text-muted">Aucune note ouverte</span>
      ) : (
        open.map((c) => {
          const active = c.check_id === activeCheckId;
          const t = checkTotals(c);
          const inProgress = c.status === "IN_PROGRESS";
          return (
            <button
              key={c.check_id}
              onClick={() => openCheck(c.check_id)}
              className={classNames(
                "press flex h-16 shrink-0 flex-col justify-center rounded-chip px-4 text-left ring-1",
                active ? "bg-blue text-white ring-transparent" : "bg-panel ring-line hover:bg-panel-2",
              )}
            >
              <span className="flex items-center gap-2 text-base font-bold">
                <span
                  className={classNames(
                    "h-2.5 w-2.5 rounded-full",
                    inProgress ? "bg-mint" : "bg-amber",
                  )}
                />
                #{c.ticket_number}
                <span className={active ? "font-medium text-white/85" : "font-medium text-muted"}>
                  {placeOf(c)}
                </span>
              </span>
              <span
                className={classNames(
                  "mt-0.5 text-sm font-semibold tabular-nums",
                  active ? "text-white/90" : "text-text",
                )}
              >
                {money(t.total, settings.currency_symbol)} · {serverOf(c.server_id)}
              </span>
            </button>
          );
        })
      )}
    </div>
  );
}
