import type { ReactNode } from "react";
import { classNames } from "@/lib/util";

/** Small uppercase label above a control — shared across all Configuration tabs. */
export function Field({ label, children }: { label: string; children: ReactNode }) {
  return (
    <label className="flex flex-col gap-1.5">
      <span className="text-xs font-semibold uppercase tracking-wide text-muted">{label}</span>
      {children}
    </label>
  );
}

/** Segmented (single-choice) control with the active option in blue. */
export function Segmented({
  value,
  options,
  onChange,
}: {
  value: string;
  options: { id: string; label: string }[];
  onChange: (id: string) => void;
}) {
  return (
    <div className="flex w-fit overflow-hidden rounded-chip ring-1 ring-line">
      {options.map((o) => (
        <button
          key={o.id}
          onClick={() => onChange(o.id)}
          className={classNames(
            "press h-10 px-4 text-sm font-semibold",
            value === o.id ? "bg-blue text-white" : "bg-panel-2 text-muted hover:text-text",
          )}
        >
          {o.label}
        </button>
      ))}
    </div>
  );
}

/** Primary "+ Add" action used at the bottom of each tab. */
export function AddButton({
  onClick,
  disabled,
  children,
}: {
  onClick: () => void;
  disabled?: boolean;
  children: ReactNode;
}) {
  return (
    <button
      onClick={onClick}
      disabled={disabled}
      className="press rounded-chip bg-blue px-5 py-3 text-sm font-bold text-white hover:opacity-90 disabled:opacity-50"
    >
      {children}
    </button>
  );
}
