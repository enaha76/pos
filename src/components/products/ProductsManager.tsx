import { useMemo } from "react";
import { useStore } from "@/store/useStore";
import { ACCENT_CHOICES, accentVar } from "@/lib/accent";
import { id, classNames } from "@/lib/util";
import { AddButton, Field } from "@/components/setup/ui";
import type { Category, Product } from "@/types/domain";

export function ProductsManager() {
  const categories = useStore((s) => s.categories);
  const products = useStore((s) => s.products);
  const settings = useStore((s) => s.settings);
  const upsertCategory = useStore((s) => s.upsertCategory);
  const upsertProduct = useStore((s) => s.upsertProduct);
  const retireProduct = useStore((s) => s.retireProduct);

  const sortedCats = useMemo(
    () => [...categories].sort((a, b) => a.display_order - b.display_order),
    [categories],
  );

  const addCategory = () => {
    const order = Math.max(0, ...categories.map((c) => c.display_order)) + 1;
    const cat: Category = {
      category_id: id("cat"),
      name: `Catégorie ${order}`,
      color: ACCENT_CHOICES[(order - 1) % ACCENT_CHOICES.length],
      display_order: order,
    };
    upsertCategory(cat);
  };

  const addProduct = () => {
    const product: Product = {
      product_id: id("p"),
      name: "Nouvel article",
      category_id: sortedCats[0]?.category_id ?? "",
      price: 0,
      active: true,
    };
    upsertProduct(product);
  };

  return (
    <div className="max-w-3xl space-y-8">
      {/* ---- categories ---- */}
      <section>
        <h2 className="mb-3 text-base font-bold">Catégories</h2>
        <div className="flex flex-col gap-3">
          {sortedCats.map((c) => (
            <div key={c.category_id} className="rounded-card bg-panel p-4 ring-1 ring-line">
              <div className="flex items-center gap-3">
                <input
                  value={c.name}
                  onChange={(e) => upsertCategory({ ...c, name: e.target.value })}
                  className="h-11 flex-1 rounded-chip bg-panel-2 px-3 text-base font-bold outline-none ring-1 ring-line focus:ring-2 focus:ring-blue"
                />
              </div>
              <div className="mt-3">
                <Field label="Couleur">
                  <div className="flex gap-2">
                    {ACCENT_CHOICES.map((color) => (
                      <button
                        key={color}
                        onClick={() => upsertCategory({ ...c, color })}
                        aria-label={color}
                        className={classNames(
                          "press h-8 w-8 rounded-full ring-2",
                          c.color === color ? "ring-text" : "ring-transparent",
                        )}
                        style={{ background: accentVar(color) }}
                      />
                    ))}
                  </div>
                </Field>
              </div>
            </div>
          ))}
        </div>
        <div className="mt-4">
          <AddButton onClick={addCategory}>+ Ajouter une catégorie</AddButton>
        </div>
      </section>

      {/* ---- products ---- */}
      <section>
        <h2 className="mb-3 text-base font-bold">Produits</h2>
        <div className="flex flex-col gap-3">
          {products.map((p) => (
            <div
              key={p.product_id}
              className={classNames(
                "rounded-card bg-panel p-4 ring-1 ring-line",
                !p.active && "opacity-60",
              )}
            >
              {/* top row: name + retire */}
              <div className="flex items-center gap-3">
                <input
                  value={p.name}
                  onChange={(e) => upsertProduct({ ...p, name: e.target.value })}
                  className="h-11 flex-1 rounded-chip bg-panel-2 px-3 text-base font-bold outline-none ring-1 ring-line focus:ring-2 focus:ring-blue"
                />
                {p.active ? (
                  <button
                    onClick={() => retireProduct(p.product_id)}
                    className="press h-11 rounded-chip px-4 text-sm font-medium text-muted hover:bg-coral/10 hover:text-coral"
                  >
                    Retirer
                  </button>
                ) : (
                  <button
                    onClick={() => upsertProduct({ ...p, active: true })}
                    className="press h-11 rounded-chip px-4 text-sm font-medium text-mint hover:bg-mint/10"
                  >
                    Restaurer
                  </button>
                )}
              </div>

              {/* settings row: category + price */}
              <div className="mt-3 flex flex-wrap items-end gap-x-8 gap-y-3">
                <Field label="Catégorie">
                  <select
                    value={p.category_id}
                    onChange={(e) => upsertProduct({ ...p, category_id: e.target.value })}
                    className="h-10 rounded-chip bg-panel-2 px-3 text-sm text-text outline-none ring-1 ring-line focus:ring-2 focus:ring-blue"
                  >
                    {sortedCats.map((c) => (
                      <option key={c.category_id} value={c.category_id}>
                        {c.name}
                      </option>
                    ))}
                  </select>
                </Field>

                <Field label="Prix">
                  <div className="flex h-10 items-center rounded-chip bg-panel-2 px-3 ring-1 ring-line focus-within:ring-2 focus-within:ring-blue">
                    <input
                      type="number"
                      step="1"
                      min="0"
                      value={String(p.price)}
                      onChange={(e) =>
                        upsertProduct({ ...p, price: Math.max(0, Math.round(Number(e.target.value) || 0)) })
                      }
                      className="w-24 bg-transparent text-right text-sm tabular-nums outline-none"
                    />
                    <span className="ml-1 text-sm text-muted">{settings.currency_symbol}</span>
                  </div>
                </Field>
              </div>
            </div>
          ))}
        </div>
        <div className="mt-4">
          <AddButton onClick={addProduct}>+ Ajouter un produit</AddButton>
        </div>
      </section>
    </div>
  );
}
