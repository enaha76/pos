import { useState } from "react";
import { useStore } from "@/store/useStore";
import { classNames } from "@/lib/util";

const DEMO_PINS: [string, string][] = [
  ["Caissier", "1111"],
  ["Propriétaire", "9999"],
];

export function Login() {
  const login = useStore((s) => s.login);
  const [pin, setPin] = useState("");
  const [error, setError] = useState(false);
  const [busy, setBusy] = useState(false);

  const press = (d: string) => {
    setError(false);
    setPin((p) => (p.length >= 6 ? p : p + d));
  };
  const back = () => {
    setError(false);
    setPin((p) => p.slice(0, -1));
  };
  const submit = async () => {
    setBusy(true);
    const ok = await login(pin); // validated server-side
    setBusy(false);
    if (!ok) {
      setError(true);
      setPin("");
    }
  };

  const keys = ["1", "2", "3", "4", "5", "6", "7", "8", "9"];

  return (
    <div className="flex h-full items-center justify-center p-6">
      <div className="w-full max-w-md rounded-card bg-panel p-10 ring-1 ring-line shadow-sm">
        <div className="text-center">
          <h1 className="text-3xl font-extrabold tracking-tight">Caisse</h1>
          <p className="mt-2 text-base text-muted">Saisissez votre code pour commencer</p>
        </div>

        {/* PIN display */}
        <div
          className={classNames(
            "mx-auto mt-7 flex h-16 items-center justify-center gap-3 rounded-chip bg-panel-2 ring-2",
            error ? "ring-coral" : "ring-line",
          )}
        >
          {pin.length === 0 ? (
            <span className="text-base text-muted">{error ? "Code incorrect — réessayez" : "Code PIN"}</span>
          ) : (
            Array.from(pin).map((_, i) => (
              <span key={i} className="h-4 w-4 rounded-full bg-text" />
            ))
          )}
        </div>

        {/* Keypad */}
        <div className="mt-6 grid grid-cols-3 gap-3">
          {keys.map((k) => (
            <button
              key={k}
              onClick={() => press(k)}
              className="press h-20 rounded-chip bg-panel-2 text-3xl font-bold ring-1 ring-line hover:bg-line"
            >
              {k}
            </button>
          ))}
          <button
            onClick={back}
            className="press h-20 rounded-chip text-lg font-semibold text-muted ring-1 ring-line hover:bg-panel-2"
          >
            Retour
          </button>
          <button
            onClick={() => press("0")}
            className="press h-20 rounded-chip bg-panel-2 text-3xl font-bold ring-1 ring-line hover:bg-line"
          >
            0
          </button>
          <button
            onClick={() => void submit()}
            disabled={busy}
            className="press h-20 rounded-chip bg-mint text-3xl font-bold text-white hover:opacity-90 disabled:opacity-60"
          >
            →
          </button>
        </div>

        {/* Demo hint */}
        {/* Dev convenience only — stripped from production builds. */}
        {import.meta.env.DEV && (
          <div className="mt-7 border-t border-line pt-5">
            <p className="text-center text-sm text-muted">Codes de démo (dev)</p>
            <div className="mt-2 flex flex-wrap justify-center gap-2">
              {DEMO_PINS.map(([name, p]) => (
                <span key={name} className="rounded-full bg-panel-2 px-4 py-1.5 text-sm text-muted">
                  {name} · {p}
                </span>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
