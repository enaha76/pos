import type { Category } from "@/types/domain";
import { accentVar } from "@/lib/accent";
import { classNames } from "@/lib/util";

interface Props {
  categories: Category[];
  active: string;
  onSelect: (id: string) => void;
}

/** Fully-rounded pills with a colored dot; active gets a blue outline (uiux-spec §5). */
export function CategoryChips({ categories, active, onSelect }: Props) {
  const chip = (id: string, label: string, dot?: string) => {
    const isActive = active === id;
    return (
      <button
        key={id}
        onClick={() => onSelect(id)}
        className={classNames(
          "press flex shrink-0 items-center gap-2 rounded-full px-5 py-2.5 text-base font-semibold ring-1",
          isActive ? "bg-blue text-white ring-transparent" : "bg-panel text-text ring-line hover:bg-panel-2",
        )}
      >
        {dot && <span className="h-3 w-3 rounded-full" style={{ background: dot }} />}
        {label}
      </button>
    );
  };

  return (
    <div className="flex gap-2 overflow-x-auto px-6 pb-3">
      {chip("all", "Tout")}
      {categories.map((c) => chip(c.category_id, c.name, accentVar(c.color)))}
    </div>
  );
}
