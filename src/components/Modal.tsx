import type { ReactNode } from "react";

interface ModalProps {
  title: string;
  subtitle?: string;
  onClose: () => void;
  children: ReactNode;
}

/** Centered card with fade backdrop (uiux-spec §5 Reason modal, §2.4 modal fade). */
export function Modal({ title, subtitle, onClose, children }: ModalProps) {
  return (
    <div
      className="fixed inset-0 z-40 flex items-center justify-center bg-black/55 p-6 animate-[fade_140ms_ease-out]"
      onClick={onClose}
    >
      <div
        className="w-full max-w-md rounded-card bg-panel p-7 ring-1 ring-line shadow-xl"
        onClick={(e) => e.stopPropagation()}
        role="dialog"
        aria-modal="true"
        aria-label={title}
      >
        <h2 className="text-2xl font-extrabold text-text">{title}</h2>
        {subtitle && <p className="mt-1.5 text-base text-muted">{subtitle}</p>}
        <div className="mt-6">{children}</div>
      </div>
      <style>{`@keyframes fade{from{opacity:0}to{opacity:1}}`}</style>
    </div>
  );
}
