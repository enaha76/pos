import { useStore } from "@/store/useStore";

export function SettingsScreen() {
  const settings = useStore((s) => s.settings);
  const updateSettings = useStore((s) => s.updateSettings);
  const bootstrap = useStore((s) => s.bootstrap);

  return (
    <div className="flex h-full flex-col">
      <header className="px-6 pt-5 pb-3">
        <h1 className="text-[21px] font-bold tracking-tight">Réglages</h1>
      </header>

      <div className="scroll-area flex-1 px-6 pb-8">
        <div className="max-w-xl space-y-4">
          <section className="rounded-card bg-panel p-5 ring-1 ring-line">
            <h2 className="text-sm font-bold">Libellé</h2>
            <p className="mt-1 text-xs text-muted">
              Le mot désignant l'emplacement où s'assoient les clients. Utilisé partout sauf si une
              zone le remplace.
            </p>
            <label className="mt-3 block text-xs font-medium text-muted">Libellé par défaut</label>
            <input
              value={settings.spot_label}
              onChange={(e) => updateSettings({ spot_label: e.target.value })}
              className="mt-1 h-11 w-48 rounded-chip bg-panel-2 px-3 text-sm outline-none ring-1 ring-line focus:ring-blue"
            />
          </section>

          <section className="rounded-card bg-panel p-5 ring-1 ring-line">
            <h2 className="text-sm font-bold">Devise</h2>
            <label className="mt-3 block text-xs font-medium text-muted">Symbole</label>
            <input
              value={settings.currency_symbol}
              onChange={(e) => updateSettings({ currency_symbol: e.target.value.slice(0, 4) })}
              className="mt-1 h-11 w-24 rounded-chip bg-panel-2 px-3 text-sm outline-none ring-1 ring-line focus:ring-blue"
            />
          </section>

          <section className="rounded-card bg-panel p-5 ring-1 ring-line">
            <h2 className="text-sm font-bold">Décisions intégrées à cette version</h2>
            <ul className="mt-2 space-y-1.5 text-sm text-muted">
              <li>• Le caissier encaisse aussi</li>
              <li>• Saisie des commandes en caisse uniquement</li>
              <li>• Annulations/offerts sans approbation — journalisés &amp; reportés</li>
              <li>• Application web installable (PWA), tablette paysage</li>
              <li>• Libellé d'emplacement configurable (par lieu / par zone)</li>
            </ul>
          </section>

          <section className="rounded-card bg-panel p-5 ring-1 ring-line">
            <h2 className="text-sm font-bold">Données serveur</h2>
            <p className="mt-1 text-xs text-muted">
              La configuration et les notes sont dans la base de données. Rechargez pour récupérer
              les dernières données du serveur.
            </p>
            <button
              onClick={() => void bootstrap()}
              className="press mt-3 rounded-chip bg-panel-2 px-4 py-2.5 text-sm font-semibold ring-1 ring-line hover:bg-line"
            >
              Recharger depuis le serveur
            </button>
          </section>
        </div>
      </div>
    </div>
  );
}
