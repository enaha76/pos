import { useMemo, useState } from "react";
import { useStore } from "@/store/useStore";
import { useToasts } from "@/store/toast";
import { Icon } from "@/components/Icon";
import { ZoneTabs } from "@/components/cashier/ZoneTabs";
import { CategoryChips } from "@/components/cashier/CategoryChips";
import { ModifierModal } from "@/components/cashier/ModifierModal";
import { accentVar } from "@/lib/accent";
import { classNames, money } from "@/lib/util";
import type { Product } from "@/types/domain";

export function MenuPanel() {
  const session = useStore((s) => s.session);
  const shifts = useStore((s) => s.shifts);
  const categories = useStore((s) => s.categories);
  const products = useStore((s) => s.products);
  const modifiers = useStore((s) => s.modifiers);
  const draft = useStore((s) => s.draft);
  const addProduct = useStore((s) => s.addProduct);
  const settings = useStore((s) => s.settings);
  const push = useToasts((s) => s.push);

  const shift = shifts.find((x) => x.shift_id === session?.shift_id);

  const [activeCat, setActiveCat] = useState<string>("all");
  const [search, setSearch] = useState("");
  const [modProduct, setModProduct] = useState<Product | null>(null);

  const modsByProduct = useMemo(() => {
    const map: Record<string, typeof modifiers> = {};
    for (const m of modifiers) (map[m.product_id] ??= []).push(m);
    return map;
  }, [modifiers]);

  const sortedCats = useMemo(
    () => [...categories].sort((a, b) => a.display_order - b.display_order),
    [categories],
  );
  const catById = useMemo(
    () => Object.fromEntries(categories.map((c) => [c.category_id, c])),
    [categories],
  );

  const visible = useMemo(() => {
    const q = search.trim().toLowerCase();
    return products
      .filter((p) => p.active)
      .filter((p) => activeCat === "all" || p.category_id === activeCat)
      .filter((p) => q === "" || p.name.toLowerCase().includes(q));
  }, [products, activeCat, search]);

  const canOrder = Boolean(session && draft.zone_id);

  const onTap = (product: Product) => {
    if (!canOrder) {
      push("Choisissez d'abord une zone");
      return;
    }
    if (!draft.server_id) {
      push("Choisissez un serveur");
      return;
    }
    if ((modsByProduct[product.product_id]?.length ?? 0) > 0) {
      setModProduct(product); // let the cashier choose modifiers first
      return;
    }
    addProduct(product.product_id);
    push(`${product.name} ajouté`); // §7 silent success
  };

  return (
    <section className="flex min-w-0 flex-1 flex-col">
      {/* Title + search */}
      <header className="flex items-center gap-4 px-6 pt-6 pb-4">
        <div className="min-w-0">
          <h1 className="truncate text-2xl font-extrabold tracking-tight">Commande</h1>
          <p className="truncate text-sm text-muted">
            {session?.name} · service {shift?.name}
          </p>
        </div>
        <div className="ml-auto flex w-72 items-center gap-2 rounded-chip bg-panel-2 px-4 ring-1 ring-line focus-within:ring-2 focus-within:ring-blue">
          <Icon name="search" className="h-5 w-5 text-muted" />
          <input
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Rechercher un plat"
            className="h-13 w-full bg-transparent py-3 text-base outline-none placeholder:text-muted"
          />
        </div>
      </header>

      <ZoneTabs />

      <CategoryChips
        categories={sortedCats}
        active={activeCat}
        onSelect={setActiveCat}
      />

      {/* Product grid */}
      <div className="scroll-area flex-1 px-6 pb-6">
        {visible.length === 0 ? (
          <p className="mt-10 text-center text-base text-muted">Aucun plat trouvé.</p>
        ) : (
          <div className="grid grid-cols-[repeat(auto-fill,minmax(180px,1fr))] gap-4">
            {visible.map((p) => {
              const cat = catById[p.category_id];
              const color = cat ? accentVar(cat.color) : "var(--color-blue)";
              return (
                <button
                  key={p.product_id}
                  onClick={() => onTap(p)}
                  disabled={!canOrder}
                  className={classNames(
                    "press relative flex min-h-[150px] flex-col justify-between overflow-hidden rounded-card bg-panel p-5 text-left ring-1 ring-line shadow-sm",
                    canOrder ? "hover:bg-panel-2" : "cursor-not-allowed opacity-50",
                  )}
                >
                  {/* colored top bar in the category color */}
                  <span
                    className="absolute inset-x-0 top-0 h-2"
                    style={{ background: color }}
                  />
                  <span className="mt-1 flex items-center gap-1.5 text-lg font-bold leading-snug">
                    {p.name}
                    {/* "options" is the same word in French */}
                    {(modsByProduct[p.product_id]?.length ?? 0) > 0 && (
                      <span className="rounded bg-panel-2 px-1.5 py-0.5 text-[11px] font-semibold text-muted">
                        options
                      </span>
                    )}
                  </span>
                  <span className="text-2xl font-extrabold" style={{ color }}>
                    {money(p.price, settings.currency_symbol)}
                  </span>
                </button>
              );
            })}
          </div>
        )}
      </div>

      {modProduct && (
        <ModifierModal
          product={modProduct}
          modifiers={modsByProduct[modProduct.product_id] ?? []}
          currency={settings.currency_symbol}
          onAdd={(ids) => {
            addProduct(modProduct.product_id, ids);
            push(`${modProduct.name} ajouté`);
            setModProduct(null);
          }}
          onClose={() => setModProduct(null)}
        />
      )}
    </section>
  );
}
