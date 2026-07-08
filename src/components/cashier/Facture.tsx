import { checkTotals } from "@/lib/totals";
import { BUSINESS_NAME, money } from "@/lib/util";
import type { Check, Server, Zone } from "@/types/domain";

interface Props {
  check: Check;
  zone?: Zone;
  tableText?: string;
  server?: Server;
  spotLabel: string;
  currency: string;
}

/**
 * Printable customer facture (l'addition) for one check.
 * Hidden on screen — shown only when `window.print()` fires (see index.css,
 * `#facture-receipt`). Works both in the browser and the Tauri desktop app.
 */
export function Facture({ check, zone, tableText, server, spotLabel, currency }: Props) {
  const totals = checkTotals(check);

  // Chargeable lines on the bill (HELD + SENT); comps are food off the bill.
  const billed = check.items.filter((it) => it.state === "HELD" || it.state === "SENT");
  const comped = check.items.filter((it) => it.state === "COMP");

  // Payment method, when the check has already been settled.
  const paidMethod = check.payments?.[0]?.method;

  const opened = new Date(check.opened_at);
  const dt = opened.toLocaleString("fr-FR", { dateStyle: "short", timeStyle: "short" });
  const printedAt = new Date().toLocaleString("fr-FR", { dateStyle: "short", timeStyle: "short" });

  return (
    <div id="facture-receipt" className="mx-auto max-w-xs text-black">
      <div className="text-center">
        <div className="text-2xl font-extrabold tracking-wide">{BUSINESS_NAME}</div>
        <div className="text-sm font-bold uppercase tracking-widest">Facture</div>
      </div>

      <Line />
      <Row k="Ticket nº" v={String(check.ticket_number)} bold />
      <Row k="Date" v={dt} />
      {zone && <Row k="Zone" v={zone.name} />}
      {tableText && <Row k={spotLabel} v={tableText} />}
      {server && <Row k="Serveur" v={server.name} />}

      <Line />
      {billed.length === 0 ? (
        <div className="text-sm">Aucun article facturé.</div>
      ) : (
        billed.map((it) => (
          <div key={it.item_id} className="flex justify-between text-sm">
            <span className="mr-2 min-w-0 flex-1 truncate">
              {it.qty} × {it.name}
            </span>
            <span className="tabular-nums">{money(it.qty * it.unit_price, currency)}</span>
          </div>
        ))
      )}

      {comped.length > 0 && (
        <>
          <div className="mt-2 mb-0.5 text-xs font-bold uppercase">Offerts (sans frais)</div>
          {comped.map((it) => (
            <div key={it.item_id} className="flex justify-between text-xs">
              <span className="mr-2 min-w-0 flex-1 truncate">
                {it.qty} × {it.name}
              </span>
              <span className="tabular-nums">0 {currency}</span>
            </div>
          ))}
        </>
      )}

      <Line />
      <Row k="Sous-total" v={money(totals.subtotal, currency)} />
      {totals.comps > 0 && <Row k="Offerts" v={`-${money(totals.comps, currency)}`} />}
      <div className="mt-1 flex justify-between text-lg font-extrabold">
        <span>TOTAL</span>
        <span className="tabular-nums">{money(totals.total, currency)}</span>
      </div>

      {paidMethod && (
        <>
          <Line />
          <Row k="Payé" v={paidMethod} bold />
        </>
      )}

      <Line />
      <div className="mt-4 text-center text-sm font-semibold">Merci de votre visite !</div>
      <div className="mt-2 text-center text-xs">Édité le {printedAt}</div>
    </div>
  );
}

function Line() {
  return <div className="my-2 border-t border-dashed border-black/40" />;
}

function Row({ k, v, bold }: { k: string; v: string; bold?: boolean }) {
  return (
    <div className={`flex justify-between text-sm${bold ? " font-bold" : ""}`}>
      <span>{k}</span>
      <span className="tabular-nums">{v}</span>
    </div>
  );
}
