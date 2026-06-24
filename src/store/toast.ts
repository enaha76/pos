import { create } from "zustand";
import { id } from "@/lib/util";

export interface ToastItem {
  id: string;
  msg: string;
}

interface ToastState {
  toasts: ToastItem[];
  push: (msg: string) => void;
  remove: (id: string) => void;
}

/** Bottom-center confirmation pills, 1.5s (uiux-spec §5 Toast / §7 silent success). */
export const useToasts = create<ToastState>((set) => ({
  toasts: [],
  push: (msg) => {
    const tid = id("toast");
    set((s) => ({ toasts: [...s.toasts, { id: tid, msg }] }));
    setTimeout(() => set((s) => ({ toasts: s.toasts.filter((t) => t.id !== tid) })), 1500);
  },
  remove: (tid) => set((s) => ({ toasts: s.toasts.filter((t) => t.id !== tid) })),
}));
