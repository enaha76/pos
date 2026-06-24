import { Modal } from "@/components/Modal";
import { PAYMENT_METHODS } from "@/data/seed";
import { money } from "@/lib/util";

interface Props {
  amount: number;
  currency: string;
  onPay: (method: string) => void;
  onClose: () => void;
}

/** Cashier settles the check (Q1 — cashier takes payment). */
export function PayModal({ amount, currency, onPay, onClose }: Props) {
  return (
    <Modal title="Encaisser" subtitle={`Montant dû ${money(amount, currency)}`} onClose={onClose}>
      <div className="flex flex-col gap-2">
        {PAYMENT_METHODS.map((m) => (
          <button
            key={m}
            onClick={() => onPay(m)}
            className="press flex items-center justify-between rounded-chip bg-panel-2 px-5 py-5 text-left text-lg font-bold ring-1 ring-line hover:bg-line"
          >
            <span>{m}</span>
            <span className="tabular-nums text-mint">{money(amount, currency)}</span>
          </button>
        ))}
      </div>
      <button
        onClick={onClose}
        className="mt-4 w-full rounded-chip px-4 py-3.5 text-base font-medium text-muted hover:text-text"
      >
        Annuler
      </button>
    </Modal>
  );
}
