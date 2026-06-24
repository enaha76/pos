import { useMemo } from "react";
import { useStore } from "@/store/useStore";
import { id, classNames } from "@/lib/util";
import { AddButton, Field, Segmented } from "@/components/setup/ui";
import type { TableMode } from "@/types/domain";

const MODES = [
  { id: "none", label: "Aucune" },
  { id: "free", label: "Libre" },
  { id: "fixed", label: "Fixe" },
];

export function ZonesManager() {
  const zones = useStore((s) => s.zones);
  const settings = useStore((s) => s.settings);
  const upsertZone = useStore((s) => s.upsertZone);
  const retireZone = useStore((s) => s.retireZone);

  const sorted = useMemo(
    () => [...zones].sort((a, b) => a.display_order - b.display_order),
    [zones],
  );

  const addZone = () => {
    const order = Math.max(0, ...zones.map((z) => z.display_order)) + 1;
    upsertZone({
      zone_id: id("zone"),
      name: `Zone ${order}`,
      display_order: order,
      table_mode: "free",
      active: true,
    });
  };

  // reorder by swapping display_order with the neighbour
  const move = (index: number, dir: -1 | 1) => {
    const a = sorted[index];
    const b = sorted[index + dir];
    if (!a || !b) return;
    upsertZone({ ...a, display_order: b.display_order });
    upsertZone({ ...b, display_order: a.display_order });
  };

  return (
    <div className="max-w-2xl">
      <p className="mb-5 text-sm text-muted">
        Les zones organisent la salle. Choisissez le mode de table — <b>Aucune</b> (sans table),{" "}
        <b>Libre</b> (numéro saisi) ou <b>Fixe</b> (tables prédéfinies). Retirer conserve
        l'historique.
      </p>

      <div className="flex flex-col gap-3">
        {sorted.map((z, i) => (
          <div
            key={z.zone_id}
            className={classNames(
              "rounded-card bg-panel p-4 ring-1 ring-line",
              !z.active && "opacity-60",
            )}
          >
            {/* top row: name · reorder · retire */}
            <div className="flex items-center gap-3">
              <input
                value={z.name}
                onChange={(e) => upsertZone({ ...z, name: e.target.value })}
                className="h-11 flex-1 rounded-chip bg-panel-2 px-3 text-base font-bold outline-none ring-1 ring-line focus:ring-2 focus:ring-blue"
              />

              <div className="flex overflow-hidden rounded-chip ring-1 ring-line">
                <button
                  onClick={() => move(i, -1)}
                  disabled={i === 0}
                  aria-label="Monter"
                  className="press h-11 w-10 bg-panel-2 text-lg leading-none hover:bg-line disabled:opacity-30"
                >
                  ↑
                </button>
                <button
                  onClick={() => move(i, 1)}
                  disabled={i === sorted.length - 1}
                  aria-label="Descendre"
                  className="press h-11 w-10 border-l border-line bg-panel-2 text-lg leading-none hover:bg-line disabled:opacity-30"
                >
                  ↓
                </button>
              </div>

              {z.active ? (
                <button
                  onClick={() => retireZone(z.zone_id)}
                  className="press h-11 rounded-chip px-4 text-sm font-medium text-muted hover:bg-coral/10 hover:text-coral"
                >
                  Retirer
                </button>
              ) : (
                <button
                  onClick={() => upsertZone({ ...z, active: true })}
                  className="press h-11 rounded-chip px-4 text-sm font-medium text-mint hover:bg-mint/10"
                >
                  Restaurer
                </button>
              )}
            </div>

            {/* settings row: table mode · spot label */}
            <div className="mt-3 flex flex-wrap items-end gap-x-8 gap-y-3">
              <Field label="Tables">
                <Segmented
                  value={z.table_mode}
                  options={MODES}
                  onChange={(m) => upsertZone({ ...z, table_mode: m as TableMode })}
                />
              </Field>

              <Field label="Mot pour l'emplacement">
                <input
                  value={z.spot_label ?? ""}
                  onChange={(e) => upsertZone({ ...z, spot_label: e.target.value || undefined })}
                  placeholder={settings.spot_label}
                  className="h-10 w-40 rounded-chip bg-panel-2 px-3 text-sm outline-none ring-1 ring-line focus:ring-2 focus:ring-blue placeholder:text-muted"
                />
              </Field>
            </div>
          </div>
        ))}
      </div>

      <div className="mt-4">
        <AddButton onClick={addZone}>+ Ajouter une zone</AddButton>
      </div>
    </div>
  );
}
