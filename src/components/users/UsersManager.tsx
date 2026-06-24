import { useEffect, useState } from "react";
import { api } from "@/lib/api";
import { classNames } from "@/lib/util";
import { AddButton, Field, Segmented } from "@/components/setup/ui";
import type { User } from "@/types/domain";

const ROLES = [
  { id: "cashier", label: "Caissier" },
  { id: "admin", label: "Admin" },
];

/** Admin screen to manage login accounts (cashier / admin). Servers have no account. */
export function UsersManager() {
  const [users, setUsers] = useState<User[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [newName, setNewName] = useState("");
  const [newRole, setNewRole] = useState<"cashier" | "admin">("cashier");
  const [newPin, setNewPin] = useState("");
  const [pinDraft, setPinDraft] = useState<Record<string, string>>({});

  const load = () =>
    api.listUsers().then(setUsers).catch((e) => setError((e as Error).message));
  useEffect(() => {
    void load();
  }, []);

  // run a mutation, surface errors, then resync from the server
  const run = async (fn: () => Promise<unknown>) => {
    setError(null);
    try {
      await fn();
    } catch (e) {
      setError((e as Error).message);
    }
    await load();
  };

  const patch = (id: string, p: Partial<User>) =>
    setUsers((list) => list.map((u) => (u.user_id === id ? { ...u, ...p } : u)));

  const onlyDigits = (s: string) => s.replace(/\D/g, "").slice(0, 6);

  return (
    <div className="max-w-3xl">
      <p className="mb-4 text-sm text-muted">
        Comptes qui se connectent à la caisse. Les serveurs (personnel de salle) n'ont pas de
        compte — ils servent uniquement à l'attribution et au planning.
      </p>

      {error && (
        <p className="mb-3 rounded-chip bg-coral/15 px-4 py-2 text-sm font-medium text-coral">
          {error}
        </p>
      )}

      {/* add account */}
      <div className="mb-5 flex flex-wrap items-end gap-x-6 gap-y-3 rounded-card bg-panel p-4 ring-1 ring-line">
        <Field label="Nom">
          <input
            value={newName}
            onChange={(e) => setNewName(e.target.value)}
            placeholder="Nom"
            className="h-10 w-40 rounded-chip bg-panel-2 px-3 text-sm text-text outline-none ring-1 ring-line focus:ring-2 focus:ring-blue placeholder:text-muted"
          />
        </Field>
        <Field label="Rôle">
          <Segmented value={newRole} options={ROLES} onChange={(r) => setNewRole(r as "cashier" | "admin")} />
        </Field>
        <Field label="Code (≥ 4 chiffres)">
          <input
            value={newPin}
            onChange={(e) => setNewPin(onlyDigits(e.target.value))}
            inputMode="numeric"
            placeholder="••••"
            className="h-10 w-28 rounded-chip bg-panel-2 px-3 text-sm text-text outline-none ring-1 ring-line focus:ring-2 focus:ring-blue placeholder:text-muted"
          />
        </Field>
        <AddButton
          disabled={!newName.trim() || newPin.length < 4}
          onClick={() =>
            run(async () => {
              await api.createUser({ name: newName, role: newRole, pin: newPin });
              setNewName("");
              setNewPin("");
              setNewRole("cashier");
            })
          }
        >
          + Ajouter
        </AddButton>
      </div>

      {/* accounts */}
      <div className="flex flex-col gap-2">
        {users.map((u) => (
          <div
            key={u.user_id}
            className={classNames("rounded-card bg-panel p-4 ring-1 ring-line", !u.active && "opacity-50")}
          >
            <div className="flex flex-wrap items-center gap-3">
              <input
                value={u.name}
                onChange={(e) => patch(u.user_id, { name: e.target.value })}
                onBlur={() => run(() => api.updateUser(u))}
                className="h-10 w-40 rounded-chip bg-panel-2 px-3 text-sm font-semibold outline-none ring-1 ring-line focus:ring-blue"
              />

              <Segmented
                value={u.role}
                options={ROLES}
                onChange={(r) => run(() => api.updateUser({ ...u, role: r }))}
              />

              {/* reset PIN */}
              <div className="flex items-center gap-1">
                <input
                  value={pinDraft[u.user_id] ?? ""}
                  onChange={(e) =>
                    setPinDraft((d) => ({ ...d, [u.user_id]: onlyDigits(e.target.value) }))
                  }
                  inputMode="numeric"
                  placeholder="Nouveau code"
                  className="h-10 w-32 rounded-chip bg-panel-2 px-3 text-sm text-text outline-none ring-1 ring-line focus:ring-blue placeholder:text-muted"
                />
                <button
                  onClick={() =>
                    run(async () => {
                      await api.setUserPin({ user_id: u.user_id, pin: pinDraft[u.user_id] ?? "" });
                      setPinDraft((d) => ({ ...d, [u.user_id]: "" }));
                    })
                  }
                  disabled={(pinDraft[u.user_id] ?? "").length < 4}
                  className="rounded-chip px-3 py-2 text-xs font-medium text-blue hover:opacity-80 disabled:opacity-40"
                >
                  Définir le code
                </button>
              </div>

              <div className="ml-auto">
                {u.active ? (
                  <button
                    onClick={() => run(() => api.updateUser({ ...u, active: false }))}
                    className="rounded-chip px-3 py-2 text-xs font-medium text-muted hover:text-coral"
                  >
                    Désactiver
                  </button>
                ) : (
                  <button
                    onClick={() => run(() => api.updateUser({ ...u, active: true }))}
                    className="rounded-chip px-3 py-2 text-xs font-medium text-mint hover:opacity-80"
                  >
                    Activer
                  </button>
                )}
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
