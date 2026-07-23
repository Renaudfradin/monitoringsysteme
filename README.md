# Monitoring Systeme

Application de bureau légère pour surveiller CPU, RAM, disque, énergie, batterie et températures en temps réel

**Stack :** Tauri v2 · React · TypeScript · Tailwind CSS · Rust

## Prérequis

- [Node.js](https://nodejs.org/) 20+
- [Rust](https://www.rust-lang.org/tools/install) (stable) — `rustup default stable`
- macOS : Xcode Command Line Tools (`xcode-select --install`)

Espace disque libre recommandé : **plusieurs Go** (toolchain Rust + crates).

## Démarrage

```bash
npm install
npm run tauri dev
```

Build production :

```bash
npm run tauri build
```

## Architecture

```
src/                  Frontend React
  components/         UI réutilisable (MetricCard, Progress, sparkline)
  pages/              Dashboard
  hooks/              useMetrics (poll 1 s)
  services/           invoke Tauri typés
  types/              Miroirs TypeScript des modèles Rust

src-tauri/            Backend Rust
  commands/           get_cpu, get_memory, get_disk, get_energy, …
  provider/           Trait SystemProvider + MacOS / Linux / Windows
  system/             Collecteurs indépendants par métrique
  models/             Structs sérialisées JSON
  cache.rs            Historique énergie circulaire (5 min @ 1 Hz)
```

### Couche d’abstraction

```rust
pub trait SystemProvider: Send + Sync {
    fn cpu(&self) -> Result<CpuMetrics, MetricError>;
    fn memory(&self) -> Result<MemoryMetrics, MetricError>;
    fn disk(&self) -> Result<DiskMetrics, MetricError>;
    fn battery(&self) -> Result<Option<BatteryMetrics>, MetricError>;
    fn energy(&self) -> Result<EnergyMetrics, MetricError>;
    fn temperature(&self) -> Result<TemperatureMetrics, MetricError>;
    fn system(&self) -> Result<SystemInfo, MetricError>;
}
```

- **V1** : `MacOSProvider` uniquement
- `LinuxProvider` / `WindowsProvider` : stubs prêts à implémenter sans toucher aux commands ni au frontend

## Énergie (macOS)

Les watts système exacts demandent souvent `powermetrics` (sudo) ou des clés SMC privées.

**V1** estime la puissance à partir de :

- baseline idle
- charge CPU × ratio de fréquence
- pression mémoire
- proxy GPU / disque / luminosité
- overhead de charge batterie

Le pourcentage est relatif à un TDP estimé selon le modèle Mac (`sysctl hw.model`).

Points d’extension documentés dans [`src-tauri/src/system/energy.rs`](src-tauri/src/system/energy.rs) pour IOKit, luminosité et `powermetrics`.

## Objectifs perf

- &lt; 1 % CPU au repos (poll 1 s, pause si fenêtre cachée)
- &lt; 100 Mo RAM
- commands async via `spawn_blocking` (pas de boucle bloquante sur le runtime UI)
- `System` sysinfo réutilisé + historique circulaire 300 points

## Roadmap (bonus)

- Graphique consommation 24 h + historique persistant
- Export CSV / JSON
- Notifications CPU / RAM &gt; 90 %
- Top processus, réseau, GPU / ventilateurs détaillés
- Widget barre de menu macOS
- Mode sombre / clair, démarrage session
- Système de plugins pour nouvelles métriques

## Licence

Open source — à préciser selon vos besoins.
