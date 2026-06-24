import { useStore, type Route } from "@/store/useStore";
import { Icon } from "@/components/Icon";
import { classNames } from "@/lib/util";

const NAV: {
  route: Route;
  icon: "cashier" | "zones" | "reports" | "settings";
  label: string;
  adminOnly: boolean;
}[] = [
  { route: "cashier", icon: "cashier", label: "Commandes", adminOnly: false },
  { route: "zones", icon: "zones", label: "Configuration", adminOnly: true },
  { route: "reports", icon: "reports", label: "Rapports", adminOnly: true },
  { route: "settings", icon: "settings", label: "Réglages", adminOnly: true },
];

export function NavRail() {
  const route = useStore((s) => s.route);
  const setRoute = useStore((s) => s.setRoute);
  const logout = useStore((s) => s.logout);
  const session = useStore((s) => s.session);
  const shifts = useStore((s) => s.shifts);

  const shift = shifts.find((x) => x.shift_id === session?.shift_id);
  const initials = session?.name.slice(0, 2).toUpperCase() ?? "";
  const isAdmin = session?.role === "admin";
  const items = NAV.filter((n) => !n.adminOnly || isAdmin);

  return (
    <nav className="flex w-[92px] shrink-0 flex-col items-center gap-2 border-r border-line bg-panel py-5">
      {/* logo */}
      <div className="mb-3 flex h-12 w-12 items-center justify-center rounded-chip bg-blue text-white">
        <Icon name="cashier" className="h-6 w-6" />
      </div>

      {items.map((n) => {
        const active = route === n.route;
        return (
          <button
            key={n.route}
            onClick={() => setRoute(n.route)}
            aria-current={active ? "page" : undefined}
            className={classNames(
              "press flex h-16 w-16 flex-col items-center justify-center gap-1 rounded-chip text-[11px] font-semibold",
              active
                ? "bg-blue text-white"
                : "text-muted hover:bg-panel-2 hover:text-text",
            )}
            title={n.label}
          >
            <Icon name={n.icon} className="h-7 w-7" />
            {n.label}
          </button>
        );
      })}

      <div className="mt-auto flex flex-col items-center gap-2">
        <button
          onClick={logout}
          className="press flex h-14 w-16 flex-col items-center justify-center gap-1 rounded-chip text-[11px] font-semibold text-muted hover:bg-panel-2 hover:text-coral"
          title="Déconnexion"
        >
          <Icon name="logout" className="h-6 w-6" />
          Quitter
        </button>
        {/* operator avatar */}
        <button
          onClick={logout}
          className="press flex h-14 w-14 flex-col items-center justify-center rounded-full bg-blue text-base font-bold text-white"
          title={`${session?.name} · service ${shift?.name}`}
        >
          {initials}
        </button>
      </div>
    </nav>
  );
}
