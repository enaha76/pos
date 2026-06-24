import { useEffect, type ReactNode } from "react";
import { useStore } from "@/store/useStore";
import { realtimeEnabled, wsUrl } from "@/lib/api";
import { Login } from "@/components/Login";
import { AppShell } from "@/components/AppShell";
import { ToastHost } from "@/components/Toast";

export function App() {
  const status = useStore((s) => s.status);
  const error = useStore((s) => s.error);
  const session = useStore((s) => s.session);
  const bootstrap = useStore((s) => s.bootstrap);

  useEffect(() => {
    void bootstrap();
  }, [bootstrap]);

  // Realtime: refetch when another station changes something (HTTP backend only).
  useEffect(() => {
    if (!realtimeEnabled) return;
    let stopped = false;
    let socket: WebSocket | null = null;
    let retry: ReturnType<typeof setTimeout> | undefined;

    const connect = () => {
      socket = new WebSocket(wsUrl());
      socket.onmessage = (ev) => {
        const s = useStore.getState();
        if (ev.data === "config") void s.refreshConfig().catch(() => {});
        void s.refreshChecks().catch(() => {});
      };
      socket.onclose = () => {
        if (!stopped) retry = setTimeout(connect, 1500); // auto-reconnect
      };
      socket.onerror = () => socket?.close();
    };

    connect();
    return () => {
      stopped = true;
      if (retry) clearTimeout(retry);
      socket?.close();
    };
  }, []);

  if (status === "loading") {
    return <Splash title="Connexion au serveur…" />;
  }
  if (status === "error") {
    return (
      <Splash
        title="Serveur injoignable"
        detail={error}
        action={
          <button
            onClick={() => void bootstrap()}
            className="press mt-6 rounded-chip bg-blue px-6 py-3 text-base font-bold text-white hover:opacity-90"
          >
            Réessayer
          </button>
        }
      />
    );
  }

  return (
    <>
      {session ? <AppShell /> : <Login />}
      <ToastHost />
    </>
  );
}

function Splash({
  title,
  detail,
  action,
}: {
  title: string;
  detail?: string;
  action?: ReactNode;
}) {
  return (
    <div className="flex h-full items-center justify-center p-6">
      <div className="max-w-md text-center">
        <h1 className="text-2xl font-extrabold tracking-tight">{title}</h1>
        {detail && <p className="mt-2 text-base text-muted">{detail}</p>}
        {action}
      </div>
    </div>
  );
}
