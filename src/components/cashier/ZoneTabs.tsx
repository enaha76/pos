import { useMemo } from "react";
import { useStore } from "@/store/useStore";
import { classNames } from "@/lib/util";

/** Rounded zone cards; active uses the blue→purple gradient with dark text (uiux-spec §5). */
export function ZoneTabs() {
  const zones = useStore((s) => s.zones);
  const draft = useStore((s) => s.draft);
  const selectZone = useStore((s) => s.selectZone);
  const settings = useStore((s) => s.settings);

  const active = useMemo(
    () => zones.filter((z) => z.active).sort((a, b) => a.display_order - b.display_order),
    [zones],
  );

  const modeCaption = (mode: string, label: string) =>
    mode === "none"
      ? "Sans table"
      : mode === "fixed"
        ? `${label}s fixes`
        : `${label} libre`;

  return (
    <div className="flex gap-2 overflow-x-auto px-6 pb-3">
      {active.map((z) => {
        const selected = draft.zone_id === z.zone_id;
        const label = z.spot_label ?? settings.spot_label;
        return (
          <button
            key={z.zone_id}
            onClick={() => selectZone(z.zone_id)}
            className={classNames(
              "press flex shrink-0 flex-col rounded-chip px-5 py-3 text-left ring-1",
              selected
                ? "bg-blue text-white ring-transparent"
                : "bg-panel text-text ring-line hover:bg-panel-2",
            )}
          >
            <span className="text-base font-bold leading-tight">{z.name}</span>
            <span
              className={classNames(
                "text-xs leading-tight",
                selected ? "text-white/80" : "text-muted",
              )}
            >
              {modeCaption(z.table_mode, label)}
            </span>
          </button>
        );
      })}
    </div>
  );
}
