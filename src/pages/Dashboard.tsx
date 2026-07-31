import { useState } from "react";
import { save } from "@tauri-apps/plugin-dialog";
import { MetricCard } from "@/components/metrics/MetricCard";
import { EnergySparkline } from "@/components/metrics/EnergySparkline";
import { Card, CardHeader, CardTitle } from "@/components/ui/card";
import { MetricProgress } from "@/components/ui/progress";
import { useMetrics } from "@/hooks/useMetrics";
import { defaultTrayDisplay, defaultVisibleSections, useSettings } from "@/hooks/useSettings";
import {
  formatBytes,
  formatDuration,
  formatRate,
  formatUptime,
} from "@/lib/utils";
import { exportMetrics, listPlugins } from "@/services/tauri";
import type {
  AppSettings,
  CpuInfo,
  DisplayInfo,
  GpuInfo,
  MemoryInfo,
  PluginInfo,
  ProcessEntry,
  SectionId,
  StorageInfo,
  SystemInfo,
  TrayDisplay,
  TrayMetricId,
  VisibleSections,
} from "@/types/metrics";

const SECTION_OPTIONS: { id: SectionId; label: string }[] = [
  { id: "cpu", label: "CPU" },
  { id: "memory", label: "RAM" },
  { id: "disk", label: "Disque" },
  { id: "energy", label: "Énergie" },
  { id: "battery", label: "Batterie" },
  { id: "temperature", label: "Température" },
  { id: "gpu", label: "GPU / Ventilateurs" },
  { id: "network", label: "Réseau" },
  { id: "processes", label: "Top processus" },
  { id: "export", label: "Export & alertes" },
  { id: "plugins", label: "Plugins métriques" },
  { id: "system", label: "Détails système" },
];

const TRAY_OPTIONS: { id: TrayMetricId; label: string; unit: string }[] = [
  { id: "cpu", label: "CPU", unit: "%" },
  { id: "memory", label: "RAM", unit: "%" },
  { id: "energy", label: "Consommation", unit: "W" },
];

export function Dashboard() {
  const { metrics, error, loading } = useMetrics();
  const { settings, update } = useSettings();
  const [range, setRange] = useState<"5m" | "24h">("5m");
  const [exporting, setExporting] = useState(false);
  const [plugins, setPlugins] = useState<PluginInfo[] | null>(null);
  const [showSections, setShowSections] = useState(false);
  const {
    cpu,
    memory,
    disk,
    energy,
    battery,
    temperature,
    processes,
    network,
    gpu,
    system,
  } = metrics;

  const sections = settings.visibleSections ?? defaultVisibleSections;
  const tray = settings.trayDisplay ?? defaultTrayDisplay;

  const machineLabel = system
    ? [system.modelName, system.model].filter(Boolean).join(" · ") || system.model
    : null;

  const chartHistory =
    range === "24h" ? (energy?.history24h ?? []) : (energy?.history ?? []);

  async function onExport(format: "json" | "csv") {
    setExporting(true);
    try {
      const path = await save({
        defaultPath: `monitoring-systeme.${format}`,
        filters: [
          {
            name: format.toUpperCase(),
            extensions: [format],
          },
        ],
      });
      if (!path) return;
      await exportMetrics(format, path);
    } catch (e) {
      console.error(e);
    } finally {
      setExporting(false);
    }
  }

  async function loadPlugins() {
    try {
      setPlugins(await listPlugins());
    } catch {
      setPlugins([]);
    }
  }

  function toggleSection(id: SectionId) {
    void update({
      visibleSections: {
        ...sections,
        [id]: !sections[id],
      },
    });
  }

  function setAllSections(visible: boolean) {
    const next = Object.fromEntries(
      SECTION_OPTIONS.map(({ id }) => [id, visible]),
    ) as VisibleSections;
    void update({ visibleSections: next });
  }

  function toggleTrayMetric(id: TrayMetricId) {
    void update({
      trayDisplay: {
        ...tray,
        [id]: !tray[id],
      },
    });
  }

  function setTrayEnabled(enabled: boolean) {
    void update({ trayDisplay: { ...tray, enabled } });
  }

  return (
    <div className="mx-auto flex min-h-full w-full max-w-md flex-col gap-3 px-4 py-5">
      <header className="mb-1 flex items-start justify-between gap-3">
        <div>
          <p className="text-[11px] font-semibold tracking-[0.14em] text-[var(--color-muted)] uppercase">
            Monitoring
          </p>
          <h1 className="mt-1 text-[28px] font-semibold tracking-tight text-[var(--color-foreground)]">
            Systeme
          </h1>
          {system ? (
            <p className="mt-1 text-[13px] text-[var(--color-muted)]">
              {machineLabel} · {system.hostname}
            </p>
          ) : (
            <p className="mt-1 text-[13px] text-[var(--color-muted)]">
              {loading ? "Chargement des métriques…" : "En attente"}
            </p>
          )}
        </div>
        <div className="flex flex-col items-end gap-1">
          <button
            type="button"
            className={`rounded-lg px-2 py-1 text-[12px] hover:bg-black/5 dark:hover:bg-white/10 ${
              showSections
                ? "bg-black/5 text-[var(--color-foreground)] dark:bg-white/10"
                : "text-[var(--color-muted)]"
            }`}
            onClick={() => setShowSections((v) => !v)}
            title="Sections affichées"
            aria-pressed={showSections}
          >
            Sections
          </button>
          <button
            type="button"
            className="rounded-lg px-2 py-1 text-[12px] text-[var(--color-muted)] hover:bg-black/5 dark:hover:bg-white/10"
            onClick={() =>
              void update({
                theme:
                  settings.theme === "dark"
                    ? "light"
                    : settings.theme === "light"
                      ? "system"
                      : "dark",
              })
            }
            title="Thème"
          >
            {settings.theme === "dark"
              ? "Sombre"
              : settings.theme === "light"
                ? "Clair"
                : "Auto"}
          </button>
        </div>
      </header>

      {error ? (
        <p className="rounded-xl bg-red-50 px-3 py-2 text-[13px] text-red-600 dark:bg-red-950/40 dark:text-red-300">
          {error}
        </p>
      ) : null}

      {showSections ? (
        <SectionsPanel
          sections={sections}
          tray={tray}
          preview={{
            cpu: cpu?.usage ?? null,
            memory: memory?.percent ?? null,
            energy: energy?.watts ?? null,
          }}
          onToggle={toggleSection}
          onToggleTray={toggleTrayMetric}
          onTrayEnabled={setTrayEnabled}
          onShowAll={() => setAllSections(true)}
          onHideAll={() => setAllSections(false)}
          onDone={() => setShowSections(false)}
        />
      ) : null}

      {sections.cpu ? (
        cpu ? (
          <MetricCard
            title="CPU"
            percent={cpu.usage}
            detail={`${cpu.coreCount} cœurs · ${cpu.frequencyMhz} MHz · load ${cpu.loadAvg.toFixed(2)}${
              system?.cpu?.brand ? ` · ${system.cpu.brand}` : ""
            }`}
            footer={
              <div className="flex flex-wrap gap-1">
                {cpu.cores.map((c, i) => (
                  <div
                    key={i}
                    className="h-1.5 min-w-[8px] flex-1 overflow-hidden rounded-full bg-black/5 dark:bg-white/10"
                    title={`Cœur ${i + 1}: ${Math.round(c)}%`}
                  >
                    <div
                      className="h-full rounded-full bg-[var(--color-accent)]/70 transition-all duration-500"
                      style={{ width: `${Math.min(100, c)}%` }}
                    />
                  </div>
                ))}
              </div>
            }
          />
        ) : (
          <SkeletonCard title="CPU" />
        )
      ) : null}

      {sections.memory ? (
        memory ? (
          <MetricCard
            title="RAM"
            percent={memory.percent}
            detail={`${formatBytes(memory.usedBytes)} utilisés · ${formatBytes(memory.totalBytes)} total${
              system?.memory?.typeName ? ` · ${system.memory.typeName}` : ""
            }`}
          />
        ) : (
          <SkeletonCard title="RAM" />
        )
      ) : null}

      {sections.disk ? (
        disk ? (
          <MetricCard
            title="Disque"
            percent={disk.percent}
            detail={`${formatBytes(disk.usedBytes)} / ${formatBytes(disk.totalBytes)}${
              system?.storage?.[0]?.model
                ? ` · ${system.storage[0].model}`
                : disk.name
                  ? ` · ${disk.name}`
                  : ""
            }`}
          />
        ) : (
          <SkeletonCard title="Disque" />
        )
      ) : null}

      {sections.energy ? (
        energy ? (
          <Card>
            <CardHeader>
              <CardTitle>Énergie</CardTitle>
              <div className="text-right">
                <div>
                  <span className="text-2xl font-semibold tracking-tight tabular-nums">
                    {energy.watts.toFixed(1)}
                  </span>
                  <span className="ml-1 text-sm text-[var(--color-muted)]">W</span>
                </div>
                <div className="text-[13px] text-[var(--color-muted)]">
                  {Math.round(energy.percent)} %
                  {energy.estimated ? " · estimé" : ""}
                </div>
              </div>
            </CardHeader>
            <MetricProgress value={energy.percent} />
            <div className="mt-3 flex gap-1">
              <RangeBtn active={range === "5m"} onClick={() => setRange("5m")}>
                5 min
              </RangeBtn>
              <RangeBtn active={range === "24h"} onClick={() => setRange("24h")}>
                24 h
              </RangeBtn>
            </div>
            <EnergySparkline
              history={chartHistory}
              label={
                range === "24h"
                  ? "Historique de consommation sur 24 heures"
                  : "Historique de consommation sur 5 minutes"
              }
            />
            <p className="mt-1 text-[12px] text-[var(--color-muted)]">
              {range === "24h"
                ? "Historique persistant · 1 point / min"
                : "Historique 5 min · rafraîchi chaque seconde"}
            </p>
          </Card>
        ) : (
          <SkeletonCard title="Énergie" />
        )
      ) : null}

      {sections.battery && battery ? (
        <MetricCard
          title="Batterie"
          percent={battery.level}
          invertTone
          detail={`${battery.state}${battery.charging ? " · en charge" : ""} · reste ${formatDuration(battery.timeRemainingSecs)}`}
        />
      ) : null}

      {sections.temperature ? (
        temperature ? (
          <Card>
            <CardHeader>
              <CardTitle>Température</CardTitle>
              {temperature.available && temperature.maxCelsius != null ? (
                <div className="text-right">
                  <span className="text-2xl font-semibold tracking-tight tabular-nums">
                    {temperature.maxCelsius.toFixed(0)}
                  </span>
                  <span className="ml-1 text-sm text-[var(--color-muted)]">°C</span>
                </div>
              ) : (
                <span className="text-[13px] text-[var(--color-muted)]">N/A</span>
              )}
            </CardHeader>
            {temperature.available ? (
              <>
                <div className="grid grid-cols-2 gap-2 text-center text-[13px] sm:grid-cols-4">
                  <TempCell label="CPU" value={temperature.cpuCelsius} />
                  <TempCell label="GPU" value={temperature.gpuCelsius} />
                  <TempCell label="SSD" value={temperature.ssdCelsius} />
                  <TempCell label="Batterie" value={temperature.batteryCelsius} />
                </div>
                {temperature.sensors.length > 0 ? (
                  <ul className="mt-3 max-h-36 space-y-1 overflow-y-auto border-t border-[var(--color-border)] pt-3 text-[12px]">
                    {temperature.sensors.slice(0, 8).map((s) => (
                      <li key={s.label} className="flex justify-between gap-2">
                        <span className="truncate text-[var(--color-muted)]">{s.label}</span>
                        <span className="shrink-0 tabular-nums">{s.celsius.toFixed(0)} °C</span>
                      </li>
                    ))}
                  </ul>
                ) : null}
                {temperature.fansRpm.length > 0 ? (
                  <p className="mt-2 text-[12px] text-[var(--color-muted)]">
                    Ventilateurs ·{" "}
                    {temperature.fansRpm.map((r) => `${Math.round(r)}`).join(" / ")} tr/min
                  </p>
                ) : null}
              </>
            ) : (
              <p className="text-[13px] text-[var(--color-muted)]">
                Capteurs thermiques indisponibles sur cette machine.
              </p>
            )}
          </Card>
        ) : loading ? (
          <SkeletonCard title="Température" />
        ) : null
      ) : null}

      {sections.gpu && gpu?.available ? (
        <Card>
          <CardHeader>
            <CardTitle>GPU / Ventilateurs</CardTitle>
            <span className="text-[13px] text-[var(--color-muted)]">{gpu.name}</span>
          </CardHeader>
          <dl className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1.5 text-[13px]">
            <InfoRow
              label="Charge"
              value={
                gpu.utilizationPercent != null
                  ? `${Math.round(gpu.utilizationPercent)} %`
                  : "N/A (Metal/IOKit)"
              }
            />
            <InfoRow
              label="Temp."
              value={
                gpu.temperatureCelsius != null
                  ? `${gpu.temperatureCelsius.toFixed(0)} °C`
                  : null
              }
            />
          </dl>
          {gpu.fans.length > 0 ? (
            <ul className="mt-3 space-y-1 border-t border-[var(--color-border)] pt-3 text-[13px]">
              {gpu.fans.map((f) => (
                <li key={f.name} className="flex justify-between gap-2">
                  <span className="text-[var(--color-muted)]">{f.name}</span>
                  <span className="tabular-nums">{Math.round(f.rpm)} tr/min</span>
                </li>
              ))}
            </ul>
          ) : (
            <p className="mt-2 text-[12px] text-[var(--color-muted)]">
              Aucun ventilateur exposé par le système
            </p>
          )}
        </Card>
      ) : null}

      {sections.network && network ? (
        <Card>
          <CardHeader>
            <CardTitle>Réseau</CardTitle>
            <span className="text-[13px] tabular-nums text-[var(--color-muted)]">
              ↓ {formatRate(network.totalRxBytesPerSec)} · ↑{" "}
              {formatRate(network.totalTxBytesPerSec)}
            </span>
          </CardHeader>
          <ul className="space-y-1.5 text-[13px]">
            {network.interfaces.slice(0, 6).map((iface) => (
              <li key={iface.name} className="flex justify-between gap-2">
                <span className="truncate text-[var(--color-muted)]">{iface.name}</span>
                <span className="shrink-0 tabular-nums">
                  ↓ {formatRate(iface.rxBytesPerSec)} · ↑{" "}
                  {formatRate(iface.txBytesPerSec)}
                </span>
              </li>
            ))}
          </ul>
        </Card>
      ) : null}

      {sections.processes && processes ? (
        <Card>
          <CardHeader>
            <CardTitle>Top processus</CardTitle>
          </CardHeader>
          <ProcessTable title="CPU" rows={processes.topCpu} mode="cpu" />
          <ProcessTable title="RAM" rows={processes.topMemory} mode="mem" />
        </Card>
      ) : null}

      {sections.export ? (
        <Card>
          <CardHeader>
            <CardTitle>Export & alertes</CardTitle>
          </CardHeader>
          <div className="flex flex-wrap gap-2">
            <ActionBtn disabled={exporting} onClick={() => void onExport("json")}>
              Export JSON
            </ActionBtn>
            <ActionBtn disabled={exporting} onClick={() => void onExport("csv")}>
              Export CSV
            </ActionBtn>
          </div>
          <label className="mt-3 flex items-center justify-between gap-3 text-[13px]">
            <span>Notifications &gt; {settings.cpuAlertThreshold} %</span>
            <input
              type="checkbox"
              checked={settings.notificationsEnabled}
              onChange={(e) =>
                void update({ notificationsEnabled: e.target.checked })
              }
            />
          </label>
          <label className="mt-2 flex items-center justify-between gap-3 text-[13px]">
            <span>Démarrer avec la session</span>
            <input
              type="checkbox"
              checked={settings.launchAtLogin}
              onChange={(e) => void update({ launchAtLogin: e.target.checked })}
            />
          </label>
          <ThresholdRow
            label="Seuil CPU"
            value={settings.cpuAlertThreshold}
            onChange={(v) => void update({ cpuAlertThreshold: v })}
          />
          <ThresholdRow
            label="Seuil RAM"
            value={settings.ramAlertThreshold}
            onChange={(v) => void update({ ramAlertThreshold: v })}
          />
        </Card>
      ) : null}

      {sections.plugins ? (
        <Card>
          <CardHeader>
            <CardTitle>Plugins métriques</CardTitle>
            <button
              type="button"
              className="text-[12px] text-[var(--color-accent)]"
              onClick={() => void loadPlugins()}
            >
              {plugins ? "Actualiser" : "Lister"}
            </button>
          </CardHeader>
          {plugins ? (
            <ul className="space-y-1.5 text-[13px]">
              {plugins.map((p) => (
                <li key={p.id} className="flex justify-between gap-2">
                  <span>
                    {p.name}
                    <span className="ml-1 text-[var(--color-muted)]">({p.id})</span>
                  </span>
                  <span className="text-[12px] text-[var(--color-muted)]">
                    {p.builtin ? "intégré" : "ext"}
                  </span>
                </li>
              ))}
            </ul>
          ) : (
            <p className="text-[12px] text-[var(--color-muted)]">
              Registre interne pour étendre les métriques sans toucher au dashboard.
            </p>
          )}
        </Card>
      ) : null}

      {sections.system && system ? <SystemDetails system={system} /> : null}
      <SettingsHint settings={settings} />
    </div>
  );
}

function SectionsPanel({
  sections,
  tray,
  preview,
  onToggle,
  onToggleTray,
  onTrayEnabled,
  onShowAll,
  onHideAll,
  onDone,
}: {
  sections: VisibleSections;
  tray: TrayDisplay;
  preview: { cpu: number | null; memory: number | null; energy: number | null };
  onToggle: (id: SectionId) => void;
  onToggleTray: (id: TrayMetricId) => void;
  onTrayEnabled: (enabled: boolean) => void;
  onShowAll: () => void;
  onHideAll: () => void;
  onDone: () => void;
}) {
  const visibleCount = SECTION_OPTIONS.filter(({ id }) => sections[id]).length;
  const trayPreview = formatTrayPreview(tray, preview);

  return (
    <Card>
      <CardHeader>
        <CardTitle>Affichage</CardTitle>
        <span className="text-[12px] text-[var(--color-muted)]">
          {visibleCount}/{SECTION_OPTIONS.length}
        </span>
      </CardHeader>

      <p className="mb-2 text-[11px] font-semibold tracking-wide text-[var(--color-muted)] uppercase">
        Barre de menu
      </p>
      <p className="mb-2 text-[12px] text-[var(--color-muted)]">
        Icône macOS et texte optionnel à côté (ex. 38% · 75% · 32W).
      </p>
      <label className="flex cursor-pointer items-center justify-between gap-3 rounded-lg px-2 py-1.5 text-[13px] hover:bg-black/5 dark:hover:bg-white/10">
        <span>Afficher dans la barre de menu</span>
        <input
          type="checkbox"
          checked={tray.enabled}
          onChange={(e) => onTrayEnabled(e.target.checked)}
        />
      </label>
      <ul className={`mt-1 space-y-1 ${tray.enabled ? "" : "opacity-45 pointer-events-none"}`}>
        {TRAY_OPTIONS.map(({ id, label, unit }) => (
          <li key={id}>
            <label
              className={`flex items-center justify-between gap-3 rounded-lg px-2 py-1.5 text-[13px] ${
                tray.enabled
                  ? "cursor-pointer hover:bg-black/5 dark:hover:bg-white/10"
                  : "cursor-default"
              }`}
            >
              <span>
                {label}
                <span className="ml-1 text-[var(--color-muted)]">({unit})</span>
              </span>
              <input
                type="checkbox"
                checked={tray[id]}
                disabled={!tray.enabled}
                onChange={() => onToggleTray(id)}
              />
            </label>
          </li>
        ))}
      </ul>
      <p className="mt-2 rounded-lg bg-black/5 px-3 py-2 text-center text-[13px] tabular-nums dark:bg-white/10">
        {!tray.enabled
          ? "Masqué de la barre de menu"
          : (trayPreview ?? "Icône seule")}
      </p>

      <p className="mt-4 mb-2 text-[11px] font-semibold tracking-wide text-[var(--color-muted)] uppercase">
        Sections du dashboard
      </p>
      <p className="mb-3 text-[12px] text-[var(--color-muted)]">
        Choisissez les blocs visibles dans la fenêtre.
      </p>
      <ul className="space-y-1">
        {SECTION_OPTIONS.map(({ id, label }) => (
          <li key={id}>
            <label className="flex cursor-pointer items-center justify-between gap-3 rounded-lg px-2 py-1.5 text-[13px] hover:bg-black/5 dark:hover:bg-white/10">
              <span>{label}</span>
              <input
                type="checkbox"
                checked={sections[id]}
                onChange={() => onToggle(id)}
              />
            </label>
          </li>
        ))}
      </ul>
      <div className="mt-3 flex flex-wrap gap-2 border-t border-[var(--color-border)] pt-3">
        <ActionBtn onClick={onShowAll}>Tout afficher</ActionBtn>
        <ActionBtn onClick={onHideAll}>Tout masquer</ActionBtn>
        <ActionBtn onClick={onDone}>Fermer</ActionBtn>
      </div>
    </Card>
  );
}

function formatTrayPreview(
  tray: TrayDisplay,
  preview: { cpu: number | null; memory: number | null; energy: number | null },
): string | null {
  if (!tray.enabled) return null;
  const parts: string[] = [];
  if (tray.cpu) {
    parts.push(
      preview.cpu != null ? `${Math.round(preview.cpu)}%` : "—%",
    );
  }
  if (tray.memory) {
    parts.push(
      preview.memory != null ? `${Math.round(preview.memory)}%` : "—%",
    );
  }
  if (tray.energy) {
    parts.push(
      preview.energy != null ? `${Math.round(preview.energy)}W` : "—W",
    );
  }
  return parts.length > 0 ? parts.join(" · ") : null;
}

function SettingsHint({ settings }: { settings: AppSettings }) {
  const trayOn = settings.trayDisplay?.enabled ?? true;
  return (
    <p className="mb-4 text-center text-[11px] text-[var(--color-muted)]">
      Thème {settings.theme}
      {trayOn
        ? " · fermer la fenêtre = barre de menu"
        : " · barre de menu masquée · fermer = quitter"}
    </p>
  );
}

function ProcessTable({
  title,
  rows,
  mode,
}: {
  title: string;
  rows: ProcessEntry[];
  mode: "cpu" | "mem";
}) {
  return (
    <div className="mt-2">
      <p className="mb-1 text-[11px] font-semibold tracking-wide text-[var(--color-muted)] uppercase">
        {title}
      </p>
      <ul className="space-y-1 text-[12px]">
        {rows.slice(0, 5).map((p) => (
          <li key={`${title}-${p.pid}`} className="flex justify-between gap-2">
            <span className="truncate">
              {p.name}
              <span className="text-[var(--color-muted)]"> · {p.pid}</span>
            </span>
            <span className="shrink-0 tabular-nums">
              {mode === "cpu"
                ? `${p.cpuPercent.toFixed(1)} %`
                : formatBytes(p.memoryBytes)}
            </span>
          </li>
        ))}
      </ul>
    </div>
  );
}

function ThresholdRow({
  label,
  value,
  onChange,
}: {
  label: string;
  value: number;
  onChange: (v: number) => void;
}) {
  return (
    <label className="mt-2 flex items-center justify-between gap-3 text-[13px]">
      <span>{label}</span>
      <input
        type="number"
        min={50}
        max={100}
        step={1}
        value={value}
        className="w-16 rounded-md border border-[var(--color-border)] bg-transparent px-2 py-0.5 text-right tabular-nums"
        onChange={(e) => onChange(Number(e.target.value))}
      />
    </label>
  );
}

function RangeBtn({
  active,
  onClick,
  children,
}: {
  active: boolean;
  onClick: () => void;
  children: React.ReactNode;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={`rounded-lg px-2.5 py-1 text-[12px] ${
        active
          ? "bg-[var(--color-accent)] text-white"
          : "bg-black/5 text-[var(--color-muted)] dark:bg-white/10"
      }`}
    >
      {children}
    </button>
  );
}

function ActionBtn({
  children,
  onClick,
  disabled,
}: {
  children: React.ReactNode;
  onClick: () => void;
  disabled?: boolean;
}) {
  return (
    <button
      type="button"
      disabled={disabled}
      onClick={onClick}
      className="rounded-lg border border-[var(--color-border)] px-3 py-1.5 text-[12px] font-medium disabled:opacity-50"
    >
      {children}
    </button>
  );
}

function SystemDetails({ system }: { system: SystemInfo }) {
  return (
    <div className="mb-2 flex flex-col gap-3">
      <Card>
        <CardHeader>
          <CardTitle>Système</CardTitle>
        </CardHeader>
        <dl className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1.5 text-[13px]">
          <InfoRow
            label="OS"
            value={system.osLongName ?? `${system.osName} ${system.osVersion}`}
          />
          <InfoRow label="Version" value={system.osVersion} />
          <InfoRow label="Build" value={system.osBuild} />
          <InfoRow label="Noyau" value={system.kernelVersion} />
          <InfoRow label="Arch" value={system.arch} />
          <InfoRow label="Uptime" value={formatUptime(system.uptimeSecs)} />
          <InfoRow label="Hôte" value={system.hostname} />
        </dl>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Machine</CardTitle>
        </CardHeader>
        <dl className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1.5 text-[13px]">
          <InfoRow label="Nom" value={system.modelName} />
          <InfoRow label="Identifiant" value={system.model} />
          <InfoRow label="Réf. modèle" value={system.modelNumber} />
          <InfoRow label="N° série" value={system.serialNumber} />
          <InfoRow label="UUID" value={system.hardwareUuid} />
          <InfoRow label="Firmware" value={system.firmwareVersion} />
        </dl>
      </Card>

      {system.cpu ? <CpuDetails cpu={system.cpu} /> : null}
      {system.memory ? <MemoryDetails memory={system.memory} /> : null}
      {system.gpu.map((g, i) => (
        <GpuDetails key={`${g.name}-${i}`} gpu={g} index={i} />
      ))}
      {system.storage.map((d, i) => (
        <StorageDetails key={`${d.bsdName ?? d.name}-${i}`} disk={d} />
      ))}
      {system.displays.map((display, i) => (
        <DisplayDetails key={`${display.name}-${i}`} display={display} />
      ))}
    </div>
  );
}

function CpuDetails({ cpu }: { cpu: CpuInfo }) {
  const topology = [
    cpu.cores != null ? `${cpu.cores} cœurs` : null,
    cpu.performanceCores != null ? `${cpu.performanceCores} P` : null,
    cpu.efficiencyCores != null ? `${cpu.efficiencyCores} E` : null,
  ]
    .filter(Boolean)
    .join(" · ");

  return (
    <Card>
      <CardHeader>
        <CardTitle>Processeur</CardTitle>
      </CardHeader>
      <dl className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1.5 text-[13px]">
        <InfoRow label="Modèle" value={cpu.brand} />
        <InfoRow label="Fabricant" value={cpu.vendor} />
        <InfoRow label="Topologie" value={topology || null} />
        <InfoRow
          label="Fréquence"
          value={cpu.frequencyMhz != null ? `${cpu.frequencyMhz} MHz` : null}
        />
      </dl>
    </Card>
  );
}

function MemoryDetails({ memory }: { memory: MemoryInfo }) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Mémoire</CardTitle>
      </CardHeader>
      <dl className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1.5 text-[13px]">
        <InfoRow
          label="Capacité"
          value={memory.totalBytes != null ? formatBytes(memory.totalBytes) : null}
        />
        <InfoRow label="Type" value={memory.typeName} />
        <InfoRow label="Fabricant" value={memory.manufacturer} />
      </dl>
      {memory.modules.length > 0 ? (
        <div className="mt-3 space-y-2 border-t border-[var(--color-border)] pt-3">
          {memory.modules.map((mod, i) => (
            <dl
              key={`${mod.slot ?? "dimm"}-${i}`}
              className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1 text-[12px]"
            >
              <InfoRow label="Slot" value={mod.slot ?? `Module ${i + 1}`} />
              <InfoRow
                label="Taille"
                value={mod.sizeBytes != null ? formatBytes(mod.sizeBytes) : null}
              />
              <InfoRow label="Type" value={mod.typeName} />
              <InfoRow
                label="Vitesse"
                value={mod.speedMhz != null ? `${mod.speedMhz} MHz` : null}
              />
              <InfoRow label="Fabricant" value={mod.manufacturer} />
              <InfoRow label="Réf." value={mod.partNumber} />
              <InfoRow label="Série" value={mod.serial} />
            </dl>
          ))}
        </div>
      ) : null}
    </Card>
  );
}

function GpuDetails({ gpu, index }: { gpu: GpuInfo; index: number }) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>{index === 0 ? "GPU" : `GPU ${index + 1}`}</CardTitle>
      </CardHeader>
      <dl className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1.5 text-[13px]">
        <InfoRow label="Modèle" value={gpu.chipset ?? gpu.name} />
        <InfoRow label="Fabricant" value={gpu.vendor} />
        <InfoRow label="Cœurs" value={gpu.cores != null ? String(gpu.cores) : null} />
        <InfoRow
          label="VRAM"
          value={gpu.vramBytes != null ? formatBytes(gpu.vramBytes) : null}
        />
        <InfoRow label="Bus" value={gpu.bus} />
        <InfoRow label="Metal" value={gpu.metalSupport} />
      </dl>
    </Card>
  );
}

function StorageDetails({ disk }: { disk: StorageInfo }) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Stockage</CardTitle>
      </CardHeader>
      <dl className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1.5 text-[13px]">
        <InfoRow label="Volume" value={disk.name} />
        <InfoRow label="Modèle" value={disk.model} />
        <InfoRow label="Type" value={disk.mediumType} />
        <InfoRow label="Protocole" value={disk.protocol} />
        <InfoRow
          label="Capacité"
          value={disk.sizeBytes != null ? formatBytes(disk.sizeBytes) : null}
        />
        <InfoRow label="BSD" value={disk.bsdName} />
        <InfoRow label="Point de montage" value={disk.mountPoint} />
        <InfoRow label="SMART" value={disk.smartStatus} />
        <InfoRow label="Série" value={disk.serial} />
      </dl>
    </Card>
  );
}

function DisplayDetails({ display }: { display: DisplayInfo }) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Écran{display.main ? " · principal" : ""}</CardTitle>
      </CardHeader>
      <dl className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1.5 text-[13px]">
        <InfoRow label="Nom" value={display.name} />
        <InfoRow label="Type" value={display.displayType} />
        <InfoRow label="Résolution" value={display.resolution} />
        <InfoRow label="Pixels" value={display.pixelResolution} />
        <InfoRow label="Connexion" value={display.connection} />
        <InfoRow
          label="Réf. produit"
          value={
            display.vendorId || display.productId
              ? `vendor ${display.vendorId ?? "—"} · product ${display.productId ?? "—"}`
              : null
          }
        />
        <InfoRow label="Série" value={display.serial} />
      </dl>
    </Card>
  );
}

function InfoRow({ label, value }: { label: string; value: string | null | undefined }) {
  if (value == null || value === "") return null;
  return (
    <>
      <dt className="text-[var(--color-muted)]">{label}</dt>
      <dd className="min-w-0 break-all text-right">{value}</dd>
    </>
  );
}

function SkeletonCard({ title }: { title: string }) {
  return (
    <Card className="opacity-70">
      <CardHeader>
        <CardTitle>{title}</CardTitle>
        <span className="text-sm text-[var(--color-muted)]">…</span>
      </CardHeader>
      <div className="h-3 w-full animate-pulse rounded-full bg-black/5 dark:bg-white/10" />
    </Card>
  );
}

function TempCell({ label, value }: { label: string; value: number | null }) {
  const tone =
    value == null
      ? "text-[var(--color-foreground)]"
      : value >= 90
        ? "text-[var(--color-metric-crit)]"
        : value >= 75
          ? "text-[var(--color-metric-warn)]"
          : "text-[var(--color-foreground)]";

  return (
    <div className="rounded-xl bg-black/[0.03] px-2 py-2 dark:bg-white/[0.06]">
      <div className="text-[11px] tracking-wide text-[var(--color-muted)] uppercase">
        {label}
      </div>
      <div className={`mt-0.5 text-base font-semibold tabular-nums ${tone}`}>
        {value != null ? `${value.toFixed(0)}°` : "—"}
      </div>
    </div>
  );
}
