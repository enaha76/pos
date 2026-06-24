import type { OrderItem } from "@/types/domain";
import { classNames, money } from "@/lib/util";

const STATE_META: Record<OrderItem["state"], { label: string; color: string; strike: boolean }> = {
  HELD: { label: "En attente", color: "text-amber", strike: false },
  SENT: { label: "Envoyé", color: "text-mint", strike: false },
  VOID: { label: "Annulé", color: "text-coral", strike: true },
  COMP: { label: "Offert", color: "text-purple", strike: true },
};

interface Props {
  item: OrderItem;
  selected: boolean;
  currency: string;
  onSelect: () => void;
}

export function OrderLineRow({ item, selected, currency, onSelect }: Props) {
  const meta = STATE_META[item.state];
  const lineTotal = item.qty * item.unit_price;
  const chargeable = item.state === "HELD" || item.state === "SENT";

  return (
    <button
      onClick={onSelect}
      className={classNames(
        "flex w-full items-center gap-3 rounded-chip px-4 py-3.5 text-left ring-1 transition-colors",
        selected ? "bg-panel-2 ring-2 ring-blue" : "ring-transparent hover:bg-panel-2",
      )}
    >
      {/* qty badge */}
      <span
        className={classNames(
          "flex h-10 min-w-10 items-center justify-center rounded-md px-2 text-lg font-bold",
          chargeable ? "bg-line text-text" : "bg-transparent text-muted",
        )}
      >
        {item.qty}
      </span>

      <span className="min-w-0 flex-1">
        <span className={classNames("block truncate text-base font-semibold", meta.strike && "line-through opacity-60")}>
          {item.name}
        </span>
        <span className={classNames("text-sm font-semibold", meta.color)}>{meta.label}</span>
      </span>

      <span
        className={classNames(
          "text-lg font-bold tabular-nums",
          meta.strike ? "text-muted line-through" : "text-text",
        )}
      >
        {money(lineTotal, currency)}
      </span>
    </button>
  );
}
