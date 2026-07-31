# Monitoring Systeme

Application de bureau légère pour surveiller CPU, RAM, disque, énergie, batterie, réseau, processus et températures en temps réel.

**Stack :** Tauri v2 · React · TypeScript · Tailwind CSS · Rust

## Prérequis

- [Node.js](https://nodejs.org/) 20+
- [Rust](https://www.rust-lang.org/tools/install) (stable) — `rustup default stable`

### macOS

- Xcode Command Line Tools (`xcode-select --install`)

### Windows

- [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) avec la charge de travail **Desktop development with C++**
- [WebView2 Runtime](https://developer.microsoft.com/microsoft-edge/webview2/) (souvent déjà installé sur Windows 10/11)
- PowerShell (inclus) pour l’inventaire hardware via CIM

Espace disque libre recommandé : **plusieurs Go** (toolchain Rust + crates).

## Démarrage

```bash
npm install
npm run tauri dev
```

Build production :

```bash
# Recommandé : builds natifs via GitHub Actions (tauri-action)
# → Windows .msi/.exe + macOS .dmg + Linux .deb/.AppImage
npm run build:ci
npm run build:watch-ci
npm run build:download-ci

# macOS uniquement en local (ce Mac)
npm run build:macos
```

Le workflow [`.github/workflows/build.yml`](.github/workflows/build.yml) tourne sur `windows-latest`, `macos-latest` et `ubuntu-22.04` — méthode officielle Tauri (pas de cross-compile depuis macOS).

Sur une machine Windows native, `npm run tauri build` produit aussi un **`.msi`** / **`.exe`** dans `src-tauri/target/release/bundle/`.

## Architecture

```
src/                  Frontend React
  components/         UI réutilisable (MetricCard, Progress, sparkline)
  pages/              Dashboard
  hooks/              useMetrics (poll 1 s), useSettings
  services/           invoke Tauri typés
  types/              Miroirs TypeScript des modèles Rust

src-tauri/            Backend Rust
  commands/           get_cpu, get_memory, export_metrics, …
  provider/           Trait SystemProvider + MacOS / Linux / Windows
  system/             Collecteurs indépendants par métrique
  plugins.rs          Registre de plugins métriques (extension point)
  models/             Structs sérialisées JSON
  cache.rs            Historique énergie 5 min + 24 h persistant
  tray.rs             Widget barre système (menu bar / tray)
  export.rs           Export CSV / JSON
  settings.rs         Thème, alertes, démarrage session
```

## Fonctionnalités

- Métriques temps réel (CPU, RAM, disque, énergie, batterie, températures)
- Graphique consommation **5 min** + **24 h** (historique JSON persistant)
- Export **CSV / JSON**
- Notifications OS si CPU / RAM &gt; seuil (défaut 90 %)
- Top processus, débit réseau, GPU / ventilateurs (best effort)
- Widget **barre système** : texte à côté de l’icône sur macOS, **tooltip** sur Windows (CPU · RAM · watts)
- Mode **sombre / clair / auto**, démarrage à la session
- **Plugins** métriques internes (`list_plugins` / `invoke_plugin`)
- Inventaire hardware : `system_profiler` (macOS) / CIM + PowerShell (Windows)

## Énergie

Les watts système exacts demandent souvent des outils privilégiés (`powermetrics` sur macOS, compteurs spécifiques sur Windows).

**V1** estime la puissance à partir de :

- baseline idle
- charge CPU × ratio de fréquence
- pression mémoire
- proxy GPU / disque / luminosité
- overhead de charge batterie

Le pourcentage est relatif à un TDP estimé selon le modèle (Mac ou PC).

Points d’extension documentés dans [`src-tauri/src/system/energy.rs`](src-tauri/src/system/energy.rs).

Sur Windows, le GPU live tente `nvidia-smi` lorsqu’il est disponible.

## Objectifs perf

- &lt; 1 % CPU au repos (poll 1 s visible, sampler fond 15 s)
- &lt; 100 Mo RAM
- commands async via `spawn_blocking` (pas de boucle bloquante sur le runtime UI)
- `System` sysinfo réutilisé + historique circulaire 300 points + 1440 points / 24 h


## Licence

Open source — à préciser selon vos besoins.
