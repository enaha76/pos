import { OpenChecksBar } from "@/components/cashier/OpenChecksBar";
import { MenuPanel } from "@/components/cashier/MenuPanel";
import { CheckPanel } from "@/components/cashier/CheckPanel";

/** The primary cashier order screen — open-checks bar on top, Menu + Check below. */
export function CashierScreen() {
  return (
    <div className="flex h-full flex-col">
      <OpenChecksBar />
      <div className="flex min-h-0 flex-1">
        <MenuPanel />
        <CheckPanel />
      </div>
    </div>
  );
}
