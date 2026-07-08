import { useMemo } from "react";
import { useStore } from "@/store/useStore";
import { id, classNames } from "@/lib/util";
import { AddButton, Field } from "@/components/setup/ui";

/**
 * Owner-managed floor staff (servers) and shifts.
 * Servers are account-less names used on checks, factures and reports;
 * shifts drive the roster and which service is active at login.
 */
export function ServersManager() {
  const servers = useStore((s) => s.servers);
  const shifts = useStore((s) => s.shifts);
  const upsertServer = useStore((s) => s.upsertServer);
  const upsertShift = useStore((s) => s.upsertShift);

  const sortedServers = useMemo(
    () => [...servers].sort((a, b) => a.name.localeCompare(b.name, "fr")),
    [servers],
  );

  const addServer = () =>
    upsertServer({ server_id: id("srv"), name: `Serveur ${servers.length + 1}`, active: true });

  const addShift = () =>
    upsertShift({ shift_id: id("shift"), name: "Nouveau service", start_time: "08:00", end_time: "16:00" });

  return (
    <div className="max-w-2xl">
      {/* ---------- staff ---------- */}
      <h2 className="mb-1 text-base font-bold">Personnel</h2>
      <p className="mb-4 text-sm text-muted">
        Les serveurs apparaissent sur les notes, les factures et les rapports. Retirer un serveur le
        masque sans effacer l'historique.
      </p>

      <div className="flex flex-col gap-2.5">
        {sortedServers.map((s) => (
          <div
            key={s.server_id}
            className={classNames(
              "flex items-center gap-3 rounded-card bg-panel p-3 ring-1 ring-line",
              !s.active && "opacity-60",
            )}
          >
            <span className="flex h-10 w-10 shrink-0 items-center justify-center rounded-full bg-blue text-sm font-bold text-white">
              {s.name.slice(0, 2).toUpperCase()}
            </span>
            <input
              value={s.name}
              onChange={(e) => upsertServer({ ...s, name: e.target.value })}
              className="h-11 flex-1 rounded-chip bg-panel-2 px-3 text-base font-bold outline-none ring-1 ring-line focus:ring-2 focus:ring-blue"
            />
            {s.active ? (
              <button
                onClick={() => upsertServer({ ...s, active: false })}
                className="press h-11 rounded-chip px-4 text-sm font-medium text-muted hover:bg-coral/10 hover:text-coral"
              >
                Retirer
              </button>
            ) : (
              <button
                onClick={() => upsertServer({ ...s, active: true })}
                className="press h-11 rounded-chip px-4 text-sm font-medium text-mint hover:bg-mint/10"
              >
                Restaurer
              </button>
            )}
          </div>
        ))}
      </div>

      <div className="mt-4">
        <AddButton onClick={addServer}>+ Ajouter un serveur</AddButton>
      </div>

      {/* ---------- shifts ---------- */}
      <h2 className="mb-1 mt-9 text-base font-bold">Services (horaires)</h2>
      <p className="mb-4 text-sm text-muted">
        Les services définissent les plages horaires. Le service actif est choisi automatiquement à
        la connexion selon l'heure.
      </p>

      <div className="flex flex-col gap-3">
        {shifts.map((sh) => (
          <div key={sh.shift_id} className="rounded-card bg-panel p-4 ring-1 ring-line">
            <div className="flex flex-wrap items-end gap-x-6 gap-y-3">
              <Field label="Nom">
                <input
                  value={sh.name}
                  onChange={(e) => upsertShift({ ...sh, name: e.target.value })}
                  className="h-10 w-44 rounded-chip bg-panel-2 px-3 text-sm font-semibold outline-none ring-1 ring-line focus:ring-2 focus:ring-blue"
                />
              </Field>
              <Field label="Début">
                <input
                  type="time"
                  value={sh.start_time}
                  onChange={(e) => upsertShift({ ...sh, start_time: e.target.value })}
                  className="h-10 rounded-chip bg-panel-2 px-3 text-sm outline-none ring-1 ring-line focus:ring-2 focus:ring-blue"
                />
              </Field>
              <Field label="Fin">
                <input
                  type="time"
                  value={sh.end_time}
                  onChange={(e) => upsertShift({ ...sh, end_time: e.target.value })}
                  className="h-10 rounded-chip bg-panel-2 px-3 text-sm outline-none ring-1 ring-line focus:ring-2 focus:ring-blue"
                />
              </Field>
            </div>
          </div>
        ))}
      </div>

      <div className="mt-4">
        <AddButton onClick={addShift}>+ Ajouter un service</AddButton>
      </div>
    </div>
  );
}
