import { useMemo, useState } from "react";
import { Modal } from "@/components/Modal";
import { classNames, money } from "@/lib/util";
import type { Modifier, Product } from "@/types/domain";

interface Props {
  product: Product;
  modifiers: Modifier[];
  currency: string;
  onAdd: (modifier_ids: string[]) => void;
  onClose: () => void;
}

/**
 * Pick modifiers before adding a product. Options sharing a group are
 * single-select (radio); ungrouped options are independent add-ons (checkbox).
 */
export function ModifierModal({ product, modifiers, currency, onAdd, onClose }: Props) {
  const [selected, setSelected] = useState<Set<string>>(new Set());

  // Split into named single-select groups + ungrouped add-ons (order preserved).
  const { groups, addons } = useMemo(() => {
    const groups: { name: string; options: Modifier[] }[] = [];
    const addons: Modifier[] = [];
    const index = new Map<string, number>();
    for (const m of modifiers) {
      if (m.mod_group) {
        let i = index.get(m.mod_group);
        if (i === undefined) {
          i = groups.length;
          index.set(m.mod_group, i);
          groups.push({ name: m.mod_group, options: [] });
        }
        groups[i].options.push(m);
      } else {
        addons.push(m);
      }
    }
    return { groups, addons };
  }, [modifiers]);

  const choose = (m: Modifier) =>
    setSelected((prev) => {
      const next = new Set(prev);
      if (m.mod_group) {
        // single-select: clear siblings, then toggle this one
        const wasSelected = next.has(m.modifier_id);
        for (const sib of modifiers) {
          if (sib.mod_group === m.mod_group) next.delete(sib.modifier_id);
        }
        if (!wasSelected) next.add(m.modifier_id);
      } else {
        next.has(m.modifier_id) ? next.delete(m.modifier_id) : next.add(m.modifier_id);
      }
      return next;
    });

  const delta = modifiers
    .filter((m) => selected.has(m.modifier_id))
    .reduce((sum, m) => sum + m.price_delta, 0);
  const total = product.price + delta;

  const option = (m: Modifier, round: boolean) => {
    const on = selected.has(m.modifier_id);
    return (
      <button
        key={m.modifier_id}
        onClick={() => choose(m)}
        className={classNames(
          "press flex items-center justify-between rounded-chip px-5 py-4 text-left text-lg font-semibold ring-1",
          on ? "bg-blue/15 text-blue ring-blue" : "bg-panel-2 text-text ring-line hover:bg-line",
        )}
      >
        <span className="flex items-center gap-3">
          <span
            className={classNames(
              "flex h-7 w-7 items-center justify-center text-sm font-bold",
              round ? "rounded-full" : "rounded-md",
              on ? "bg-blue text-white" : "bg-line text-muted",
            )}
          >
            {on ? "✓" : round ? "" : "+"}
          </span>
          {m.name}
        </span>
        {m.price_delta !== 0 && (
          <span className="tabular-nums text-muted">
            {m.price_delta > 0 ? "+" : ""}
            {money(m.price_delta, currency)}
          </span>
        )}
      </button>
    );
  };

  return (
    <Modal title={product.name} subtitle="Choisissez les options, puis ajoutez" onClose={onClose}>
      <div className="flex flex-col gap-4">
        {groups.map((g) => (
          <div key={g.name}>
            <div className="mb-1.5 text-sm font-bold uppercase tracking-wide text-muted">
              {g.name} <span className="font-medium normal-case">· choisir un</span>
            </div>
            <div className="flex flex-col gap-2">{g.options.map((m) => option(m, true))}</div>
          </div>
        ))}

        {addons.length > 0 && (
          <div>
            {groups.length > 0 && (
              <div className="mb-1.5 text-sm font-bold uppercase tracking-wide text-muted">
                Suppléments
              </div>
            )}
            <div className="flex flex-col gap-2">{addons.map((m) => option(m, false))}</div>
          </div>
        )}
      </div>

      <button
        onClick={() => onAdd([...selected])}
        className="press mt-5 flex h-14 w-full items-center justify-center gap-3 rounded-chip bg-mint text-lg font-bold text-white hover:opacity-90"
      >
        Ajouter · {money(total, currency)}
      </button>
      <button
        onClick={onClose}
        className="mt-2 w-full rounded-chip px-4 py-3 text-base font-medium text-muted hover:text-text"
      >
        Annuler
      </button>
    </Modal>
  );
}
