import { useMemo } from "react";
import { useStore } from "@/store/useStore";
import { Modal } from "@/components/Modal";
import type { ReasonKind } from "@/types/domain";

interface Props {
  kind: ReasonKind;
  title: string;
  subtitle: string;
  onPick: (reason_id: string) => void;
  onClose: () => void;
}

/** Reason picker for void / comp / unpaid-close (uiux-spec §5, cashier-charge §8). */
export function ReasonModal({ kind, title, subtitle, onPick, onClose }: Props) {
  // Select the stable array, then filter — selecting a fresh array each render
  // would loop with useSyncExternalStore.
  const reasonCodes = useStore((s) => s.reasonCodes);
  const reasons = useMemo(
    () => reasonCodes.filter((r) => r.active && r.kind === kind),
    [reasonCodes, kind],
  );

  return (
    <Modal title={title} subtitle={subtitle} onClose={onClose}>
      <div className="flex flex-col gap-2">
        {reasons.map((r) => (
          <button
            key={r.reason_id}
            onClick={() => onPick(r.reason_id)}
            className="press rounded-chip bg-panel-2 px-5 py-4 text-left text-base font-semibold ring-1 ring-line hover:bg-line"
          >
            {r.label}
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
