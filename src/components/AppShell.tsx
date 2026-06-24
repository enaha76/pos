import { useStore } from "@/store/useStore";
import { NavRail } from "@/components/NavRail";
import { CashierScreen } from "@/components/cashier/CashierScreen";
import { SetupScreen } from "@/components/setup/SetupScreen";
import { ReportsScreen } from "@/components/reports/ReportsScreen";
import { SettingsScreen } from "@/components/settings/SettingsScreen";

export function AppShell() {
  const route = useStore((s) => s.route);
  const isAdmin = useStore((s) => s.session?.role === "admin");

  // Cashiers only ever see the order screen; admin screens are role-gated
  // (the backend enforces this too).
  return (
    <div className="flex h-full">
      <NavRail />
      <main className="flex-1 overflow-hidden">
        {!isAdmin || route === "cashier" ? (
          <CashierScreen />
        ) : route === "zones" || route === "products" ? (
          <SetupScreen />
        ) : route === "reports" ? (
          <ReportsScreen />
        ) : (
          <SettingsScreen />
        )}
      </main>
    </div>
  );
}
