import { useMemo, useState } from "react";
import { useStore } from "@/store/useStore";
import { classNames, todayStr } from "@/lib/util";
import { Field, Segmented } from "@/components/setup/ui";

/** Roster — assign servers to zones per shift, per day (§9 ShiftAssignment). */
export function ShiftAssignmentManager() {
  const zones = useStore((s) => s.zones);
  const servers = useStore((s) => s.servers);
  const shifts = useStore((s) => s.shifts);
  const assignments = useStore((s) => s.shiftAssignments);
  const toggleAssignment = useStore((s) => s.toggleAssignment);

  const [date, setDate] = useState(todayStr());
  const [shiftId, setShiftId] = useState(shifts[0]?.shift_id ?? "");

  const activeZones = useMemo(
    () => zones.filter((z) => z.active).sort((a, b) => a.display_order - b.display_order),
    [zones],
  );
  const activeServers = useMemo(() => servers.filter((s) => s.active), [servers]);

  const isAssigned = (server_id: string, zone_id: string) =>
    assignments.some(
      (a) =>
        a.server_id === server_id &&
        a.zone_id === zone_id &&
        a.shift_id === shiftId &&
        a.date === date,
    );

  return (
    <div className="max-w-3xl">
      <p className="mb-4 text-sm text-muted">
        Affectez quel serveur travaille dans quelle zone, par service et par jour. Le sélecteur de
        serveur en caisse est filtré selon ces affectations. Si une zone n'a personne d'affecté,
        tout serveur peut être choisi.
      </p>

      {/* controls */}
      <div className="mb-5 flex flex-wrap items-end gap-x-8 gap-y-3 rounded-card bg-panel p-4 ring-1 ring-line">
        <Field label="Date">
          <input
            type="date"
            value={date}
            onChange={(e) => setDate(e.target.value || todayStr())}
            className="h-10 rounded-chip bg-panel-2 px-3 text-sm text-text outline-none ring-1 ring-line focus:ring-2 focus:ring-blue"
          />
        </Field>
        <Field label="Service">
          <Segmented
            value={shiftId}
            options={shifts.map((sh) => ({
              id: sh.shift_id,
              label: `${sh.name} (${sh.start_time}–${sh.end_time})`,
            }))}
            onChange={setShiftId}
          />
        </Field>
      </div>

      {/* zone rows */}
      <div className="flex flex-col gap-3">
        {activeZones.map((z) => (
          <div key={z.zone_id} className="rounded-card bg-panel p-4 ring-1 ring-line">
            <div className="mb-3 text-base font-bold">{z.name}</div>
            <div className="flex flex-wrap gap-2">
              {activeServers.map((s) => {
                const on = isAssigned(s.server_id, z.zone_id);
                return (
                  <button
                    key={s.server_id}
                    onClick={() => toggleAssignment(s.server_id, z.zone_id, shiftId, date)}
                    className={classNames(
                      "press flex items-center gap-2 rounded-full px-3 py-2 text-sm font-medium ring-1",
                      on
                        ? "bg-blue/15 text-blue ring-blue"
                        : "bg-panel-2 text-muted ring-line hover:text-text",
                    )}
                  >
                    <span
                      className={classNames(
                        "flex h-6 w-6 items-center justify-center rounded-full text-[10px] font-bold",
                        on ? "bg-blue text-white" : "bg-line text-text",
                      )}
                    >
                      {s.name.slice(0, 2).toUpperCase()}
                    </span>
                    {s.name}
                  </button>
                );
              })}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
