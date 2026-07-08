import { useState } from "react";
import { useStore } from "@/store/useStore";
import { useToasts } from "@/store/toast";
import { Icon } from "@/components/Icon";
import { OrderLineRow } from "@/components/cashier/OrderLineRow";
import { ReasonModal } from "@/components/cashier/ReasonModal";
import { PayModal } from "@/components/cashier/PayModal";
import { Facture } from "@/components/cashier/Facture";
import { Modal } from "@/components/Modal";
import { checkTotals } from "@/lib/totals";
import { classNames, money, todayStr } from "@/lib/util";
import type { CheckStatus } from "@/types/domain";

const STATUS_TAG: Record<CheckStatus, { label: string; cls: string }> = {
  OPEN: { label: "Ouvert", cls: "bg-panel-2 text-muted" },
  IN_PROGRESS: { label: "En cours", cls: "bg-blue/15 text-blue" },
  CLOSED_PAID: { label: "Payé", cls: "bg-mint/15 text-mint" },
  CLOSED_UNPAID: { label: "Impayé", cls: "bg-coral/15 text-coral" },
  VOIDED: { label: "Annulé", cls: "bg-coral/15 text-coral" },
};

type ModalState =
  | { kind: "void"; item_id: string }
  | { kind: "comp"; item_id: string }
  | { kind: "unpaid" }
  | { kind: "pay" }
  | { kind: "server" }
  | null;

export function CheckPanel() {
  const checks = useStore((s) => s.checks);
  const activeCheckId = useStore((s) => s.activeCheckId);
  const draft = useStore((s) => s.draft);
  const zones = useStore((s) => s.zones);
  const tableSpots = useStore((s) => s.tableSpots);
  const servers = useStore((s) => s.servers);
  const setOrderServer = useStore((s) => s.setOrderServer);
  const session = useStore((s) => s.session);
  const shiftAssignments = useStore((s) => s.shiftAssignments);
  const settings = useStore((s) => s.settings);
  const selectedItemId = useStore((s) => s.selectedItemId);
  const selectItem = useStore((s) => s.selectItem);
  const setTable = useStore((s) => s.setTable);
  const incLine = useStore((s) => s.incLine);
  const send = useStore((s) => s.send);
  const voidLine = useStore((s) => s.voidLine);
  const compLine = useStore((s) => s.compLine);
  const pay = useStore((s) => s.pay);
  const closeUnpaid = useStore((s) => s.closeUnpaid);
  const push = useToasts((s) => s.push);

  const [modal, setModal] = useState<ModalState>(null);

  const check = checks.find((c) => c.check_id === activeCheckId) ?? null;
  const zone = zones.find((z) => z.zone_id === draft.zone_id);
  const totals = checkTotals(check);
  const cur = settings.currency_symbol;
  const spotLabel = zone?.spot_label ?? settings.spot_label;

  // responsible floor server for this order (cashier-centric — assignable per check)
  const orderServerId = check?.server_id ?? draft.server_id;
  const orderServer = servers.find((s) => s.server_id === orderServerId);

  // §6.2 — picker is filtered to servers rostered to this zone for the current shift.
  const rosteredIds = new Set(
    shiftAssignments
      .filter(
        (a) =>
          a.zone_id === draft.zone_id &&
          a.shift_id === session?.shift_id &&
          a.date === todayStr(),
      )
      .map((a) => a.server_id),
  );
  const hasRoster = rosteredIds.size > 0;
  const pickerServers = servers.filter(
    (s) => s.active && (!hasRoster || rosteredIds.has(s.server_id)),
  );

  const selected = check?.items.find((it) => it.item_id === selectedItemId) ?? null;

  const tableText =
    check?.table_label ??
    tableSpots.find((t) => t.table_id === check?.table_id)?.label ??
    draft.table_label ??
    tableSpots.find((t) => t.table_id === draft.table_id)?.label;

  const onSend = () => {
    const n = totals.heldQty;
    send();
    push(`${n} article${n === 1 ? "" : "s"} envoyé${n === 1 ? "" : "s"} en cuisine`);
  };

  return (
    <aside className="flex w-[400px] shrink-0 flex-col border-l border-line bg-panel">
      {/* ---- header ---- */}
      <div className="border-b border-line px-5 pt-5 pb-4">
        <div className="flex items-center justify-between">
          <span className="text-xl font-extrabold text-amber tabular-nums">
            {check ? `Ticket nº ${check.ticket_number}` : "Nouvelle note"}
          </span>
          <span
            className={classNames(
              "rounded-full px-3 py-1 text-sm font-bold",
              STATUS_TAG[check?.status ?? "OPEN"].cls,
            )}
          >
            {STATUS_TAG[check?.status ?? "OPEN"].label}
          </span>
        </div>

        {/* zone + table line */}
        <div className="mt-1.5 text-sm font-medium text-muted">
          {zone ? zone.name : "Aucune zone sélectionnée"}
          {zone && zone.table_mode !== "none" && tableText ? ` · ${spotLabel} ${tableText}` : ""}
        </div>

        {/* server assignment — which floor server this order belongs to */}
        <button
          onClick={() => setModal({ kind: "server" })}
          className="press mt-3 flex w-full items-center gap-3 rounded-chip bg-panel-2 px-4 py-3 text-left ring-1 ring-line hover:bg-line"
        >
          <span className="flex h-9 w-9 items-center justify-center rounded-full bg-blue text-xs font-bold text-white">
            {orderServer ? orderServer.name.slice(0, 2).toUpperCase() : "?"}
          </span>
          <span className="text-base">
            <span className="text-muted">Serveur </span>
            <span className="font-bold text-text">
              {orderServer ? orderServer.name : "Assigner…"}
            </span>
          </span>
          <span className="ml-auto text-muted">▾</span>
        </button>

        {/* table field — hidden when mode = none */}
        {zone && zone.table_mode === "free" && (
          <input
            value={draft.table_label ?? ""}
            onChange={(e) => setTable(undefined, e.target.value || undefined)}
            placeholder={`Nº de ${spotLabel.toLowerCase()} (optionnel)`}
            className="mt-3 h-12 w-full rounded-chip bg-panel-2 px-4 text-base outline-none ring-1 ring-line focus:ring-2 focus:ring-blue placeholder:text-muted"
          />
        )}
        {zone && zone.table_mode === "fixed" && (
          <div className="mt-3 flex flex-wrap gap-2">
            {tableSpots
              .filter((t) => t.zone_id === zone.zone_id && t.active)
              .map((t) => {
                const on = draft.table_id === t.table_id;
                return (
                  <button
                    key={t.table_id}
                    onClick={() => setTable(t.table_id, undefined)}
                    className={classNames(
                      "press h-12 min-w-12 rounded-chip px-3 text-base font-bold ring-1",
                      on ? "bg-blue text-white ring-transparent" : "bg-panel-2 text-text ring-line hover:bg-line",
                    )}
                  >
                    {t.label}
                  </button>
                );
              })}
          </div>
        )}
      </div>

      {/* ---- order lines ---- */}
      <div className="scroll-area flex-1 px-2 py-2">
        {!check || check.items.length === 0 ? (
          <div className="flex h-full items-center justify-center px-6 text-center">
            <p className="text-base text-muted">Aucun article. Touchez un plat pour commencer.</p>
          </div>
        ) : (
          <div className="flex flex-col gap-1">
            {check.items.map((it) => (
              <OrderLineRow
                key={it.item_id}
                item={it}
                currency={cur}
                selected={selectedItemId === it.item_id}
                onSelect={() => selectItem(selectedItemId === it.item_id ? null : it.item_id)}
              />
            ))}
          </div>
        )}
      </div>

      {/* ---- contextual line actions ---- */}
      {selected && (selected.state === "HELD" || selected.state === "SENT") && (
        <div className="border-t border-line px-4 py-3">
          {selected.state === "HELD" ? (
            <div className="flex items-center gap-3">
              <div className="flex items-center gap-1 rounded-chip bg-panel-2 p-1 ring-1 ring-line">
                <button
                  onClick={() => incLine(selected.item_id, -1)}
                  className="press flex h-12 w-12 items-center justify-center rounded-md hover:bg-line"
                  aria-label="Decrease quantity"
                >
                  <Icon name="minus" className="h-5 w-5" />
                </button>
                <span className="w-9 text-center text-lg font-bold tabular-nums">{selected.qty}</span>
                <button
                  onClick={() => incLine(selected.item_id, 1)}
                  className="press flex h-12 w-12 items-center justify-center rounded-md hover:bg-line"
                  aria-label="Increase quantity"
                >
                  <Icon name="plus" className="h-5 w-5" />
                </button>
              </div>
              <button
                onClick={() => setModal({ kind: "void", item_id: selected.item_id })}
                className="press ml-auto h-12 rounded-chip bg-coral px-6 text-base font-bold text-white hover:opacity-90"
              >
                Annuler
              </button>
            </div>
          ) : (
            <button
              onClick={() => setModal({ kind: "comp", item_id: selected.item_id })}
              className="press h-12 w-full rounded-chip bg-purple text-base font-bold text-white hover:opacity-90"
            >
              Offrir
            </button>
          )}
        </div>
      )}

      {/* ---- totals + actions ---- */}
      <div className="border-t border-line px-5 py-4">
        <div className="space-y-1.5 text-base">
          <Row label="Sous-total" value={money(totals.subtotal, cur)} muted />
          {totals.comps > 0 && (
            <Row label="Offerts" value={`-${money(totals.comps, cur)}`} className="text-purple" />
          )}
          <div className="flex items-center justify-between pt-1">
            <span className="text-lg font-bold">Total</span>
            <span className="text-4xl font-extrabold tabular-nums text-text">
              {money(totals.total, cur)}
            </span>
          </div>
        </div>

        <div className="mt-4 grid grid-cols-2 gap-3">
          <button
            onClick={onSend}
            disabled={!totals.canSend}
            className={classNames(
              "h-16 rounded-chip text-lg font-bold",
              totals.canSend ? "press bg-mint text-white hover:opacity-90" : "cursor-not-allowed bg-mint/30 text-white/70",
            )}
          >
            Envoyer
          </button>
          <button
            onClick={() => setModal({ kind: "pay" })}
            disabled={!totals.canPay}
            className={classNames(
              "h-16 rounded-chip text-lg font-bold",
              totals.canPay
                ? "press bg-blue text-white hover:opacity-90"
                : "cursor-not-allowed bg-blue/30 text-white/70",
            )}
          >
            Payer
          </button>
        </div>

        {check && totals.total > 0 && (
          <button
            onClick={() => window.print()}
            className="press mt-3 flex w-full items-center justify-center gap-2 rounded-chip bg-panel-2 py-3 text-base font-bold text-text ring-1 ring-line hover:bg-line"
          >
            <Icon name="printer" className="h-5 w-5" />
            Imprimer la facture
          </button>
        )}

        {check && (check.status === "OPEN" || check.status === "IN_PROGRESS") && totals.total > 0 && (
          <button
            onClick={() => setModal({ kind: "unpaid" })}
            className="mt-3 w-full rounded-chip py-2.5 text-sm font-medium text-muted hover:text-coral"
          >
            Clôturer impayé (départ / perte)
          </button>
        )}
      </div>

      {/* Printable facture (hidden on screen; shown only when printing). */}
      {check && (
        <Facture
          check={check}
          zone={zone}
          tableText={tableText}
          server={orderServer}
          spotLabel={spotLabel}
          currency={cur}
        />
      )}

      {/* ---- modals ---- */}
      {modal?.kind === "void" && (
        <ReasonModal
          kind="void"
          title="Annuler l'article"
          subtitle="Retiré avant préparation — sans frais."
          onPick={(r) => {
            voidLine(modal.item_id, r);
            push("Article annulé");
            setModal(null);
          }}
          onClose={() => setModal(null)}
        />
      )}
      {modal?.kind === "comp" && (
        <ReasonModal
          kind="comp"
          title="Offrir l'article"
          subtitle="Retiré après préparation — coût enregistré."
          onPick={(r) => {
            compLine(modal.item_id, r);
            push("Article offert");
            setModal(null);
          }}
          onClose={() => setModal(null)}
        />
      )}
      {modal?.kind === "unpaid" && (
        <ReasonModal
          kind="unpaid"
          title="Clôturer la note impayée"
          subtitle="Signalé au propriétaire et attribué au serveur."
          onPick={(r) => {
            closeUnpaid(r);
            push("Note clôturée impayée");
            setModal(null);
          }}
          onClose={() => setModal(null)}
        />
      )}
      {modal?.kind === "server" && (
        <Modal
          title="Assigner un serveur"
          subtitle={
            hasRoster
              ? "Serveurs affectés à cette zone pour ce service."
              : "Aucune affectation — tout serveur peut être choisi."
          }
          onClose={() => setModal(null)}
        >
          <div className="flex flex-col gap-2">
            {pickerServers.map((s) => {
                const current = orderServerId === s.server_id;
                return (
                  <button
                    key={s.server_id}
                    onClick={() => {
                      setOrderServer(s.server_id);
                      push(`Assigné à ${s.name}`);
                      setModal(null);
                    }}
                    className={classNames(
                      "press flex items-center justify-between rounded-chip px-5 py-4 text-left text-lg font-semibold ring-1",
                      current ? "bg-panel-2 ring-2 ring-blue" : "bg-panel-2 ring-line hover:bg-line",
                    )}
                  >
                    <span className="flex items-center gap-3">
                      <span className="flex h-9 w-9 items-center justify-center rounded-full bg-blue text-sm font-bold text-white">
                        {s.name.slice(0, 2).toUpperCase()}
                      </span>
                      {s.name}
                    </span>
                    {current && <span className="text-sm font-bold text-blue">Actuel</span>}
                  </button>
                );
              })}
          </div>
          <button
            onClick={() => setModal(null)}
            className="mt-4 w-full rounded-chip px-4 py-3.5 text-base font-medium text-muted hover:text-text"
          >
            Annuler
          </button>
        </Modal>
      )}
      {modal?.kind === "pay" && (
        <PayModal
          amount={totals.total}
          currency={cur}
          onPay={(m) => {
            pay(m);
            push("Payé — note clôturée");
            setModal(null);
          }}
          onClose={() => setModal(null)}
        />
      )}
    </aside>
  );
}

function Row({
  label,
  value,
  muted,
  className,
}: {
  label: string;
  value: string;
  muted?: boolean;
  className?: string;
}) {
  return (
    <div className="flex items-center justify-between">
      <span className={muted ? "text-muted" : ""}>{label}</span>
      <span className={classNames("tabular-nums", className)}>{value}</span>
    </div>
  );
}
