import { useToasts } from "@/store/toast";

/** Bottom-center toast stack with slide-up motion (uiux-spec §2.4 / §5). */
export function ToastHost() {
  const toasts = useToasts((s) => s.toasts);
  return (
    <div className="pointer-events-none fixed inset-x-0 bottom-6 z-50 flex flex-col items-center gap-2">
      {toasts.map((t) => (
        <div
          key={t.id}
          className="animate-[toast_180ms_ease-out] rounded-chip bg-text px-6 py-3.5 text-base font-semibold text-panel shadow-xl"
        >
          {t.msg}
        </div>
      ))}
      <style>{`@keyframes toast{from{opacity:0;transform:translateY(12px)}to{opacity:1;transform:translateY(0)}}`}</style>
    </div>
  );
}
