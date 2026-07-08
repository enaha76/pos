import { useState } from "react";
import { ZonesManager } from "@/components/zones/ZonesManager";
import { ProductsManager } from "@/components/products/ProductsManager";
import { ServersManager } from "@/components/servers/ServersManager";
import { ShiftAssignmentManager } from "@/components/roster/ShiftAssignmentManager";
import { UsersManager } from "@/components/users/UsersManager";
import { classNames } from "@/lib/util";

type Tab = "zones" | "products" | "staff" | "roster" | "users";

const TABS: { id: Tab; label: string }[] = [
  { id: "zones", label: "Zones" },
  { id: "products", label: "Produits & catégories" },
  { id: "staff", label: "Serveurs" },
  { id: "roster", label: "Affectations" },
  { id: "users", label: "Utilisateurs" },
];

/** Owner/admin setup — zones (§4.2) and products & categories (§4.3). */
export function SetupScreen() {
  const [tab, setTab] = useState<Tab>("zones");
  return (
    <div className="flex h-full flex-col">
      <header className="flex items-center gap-2 px-6 pt-5 pb-3">
        <h1 className="text-[21px] font-bold tracking-tight">Configuration</h1>
        <div className="ml-4 flex gap-1 rounded-chip bg-panel-2 p-1 ring-1 ring-line">
          {TABS.map((t) => (
            <button
              key={t.id}
              onClick={() => setTab(t.id)}
              className={classNames(
                "rounded-md px-4 py-1.5 text-sm font-medium",
                tab === t.id ? "bg-panel text-text ring-1 ring-line" : "text-muted hover:text-text",
              )}
            >
              {t.label}
            </button>
          ))}
        </div>
      </header>
      <div className="scroll-area flex-1 px-6 pb-8">
        {tab === "zones" && <ZonesManager />}
        {tab === "products" && <ProductsManager />}
        {tab === "staff" && <ServersManager />}
        {tab === "roster" && <ShiftAssignmentManager />}
        {tab === "users" && <UsersManager />}
      </div>
    </div>
  );
}
