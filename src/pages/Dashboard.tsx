import { MetricCard } from "@/components/metrics/MetricCard";
import { EnergySparkline } from "@/components/metrics/EnergySparkline";
import { Card, CardHeader, CardTitle } from "@/components/ui/card";
import { MetricProgress } from "@/components/ui/progress";
import { useMetrics } from "@/hooks/useMetrics";
import {
  formatBytes,
  formatDuration,
  formatUptime,
} from "@/lib/utils";

export function Dashboard() {
  const { metrics, error, loading } = useMetrics();
  const { cpu, memory, disk, energy, battery, temperature, system } = metrics;

  return (
    <div className="mx-auto flex min-h-full w-full max-w-md flex-col gap-3 px-4 py-5">
      <header className="mb-1">
        <p className="text-[11px] font-semibold tracking-[0.14em] text-[var(--color-muted)] uppercase">
          Monitoring
        </p>
        <h1 className="mt-1 text-[28px] font-semibold tracking-tight text-[var(--color-foreground)]">
          Systeme
        </h1>
        {system ? (
          <p className="mt-1 text-[13px] text-[var(--color-muted)]">
            {system.model} · {system.hostname}
          </p>
        ) : (
          <p className="mt-1 text-[13px] text-[var(--color-muted)]">
            {loading ? "Chargement des métriques…" : "En attente"}
          </p>
        )}
      </header>

      {error ? (
        <p className="rounded-xl bg-red-50 px-3 py-2 text-[13px] text-red-600">
          {error}
        </p>
      ) : null}

      {cpu ? (
        <MetricCard
          title="CPU"
          percent={cpu.usage}
          detail={`${cpu.coreCount} cœurs · ${cpu.frequencyMhz} MHz · load ${cpu.loadAvg.toFixed(2)}`}
          footer={
            <div className="flex flex-wrap gap-1">
              {cpu.cores.map((c, i) => (
                <div
                  key={i}
                  className="h-1.5 flex-1 min-w-[8px] overflow-hidden rounded-full bg-black/5"
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
      )}

      {memory ? (
        <MetricCard
          title="RAM"
          percent={memory.percent}
          detail={`${formatBytes(memory.usedBytes)} utilisés · ${formatBytes(memory.totalBytes)} total`}
        />
      ) : (
        <SkeletonCard title="RAM" />
      )}

      {disk ? (
        <MetricCard
          title="Disque"
          percent={disk.percent}
          detail={`${formatBytes(disk.usedBytes)} / ${formatBytes(disk.totalBytes)}${disk.name ? ` · ${disk.name}` : ""}`}
        />
      ) : (
        <SkeletonCard title="Disque" />
      )}

      {energy ? (
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
          <EnergySparkline history={energy.history} />
          <p className="mt-1 text-[12px] text-[var(--color-muted)]">
            Historique 5 min · rafraîchi chaque seconde
          </p>
        </Card>
      ) : (
        <SkeletonCard title="Énergie" />
      )}

      {battery ? (
        <MetricCard
          title="Batterie"
          percent={battery.level}
          detail={`${battery.state}${battery.charging ? " · en charge" : ""} · reste ${formatDuration(battery.timeRemainingSecs)}`}
        />
      ) : null}

      {temperature?.available ? (
        <Card>
          <CardHeader>
            <CardTitle>Température</CardTitle>
          </CardHeader>
          <div className="grid grid-cols-3 gap-2 text-center text-[13px]">
            <TempCell label="CPU" value={temperature.cpuCelsius} />
            <TempCell label="GPU" value={temperature.gpuCelsius} />
            <TempCell label="SSD" value={temperature.ssdCelsius} />
          </div>
        </Card>
      ) : null}

      {system ? (
        <Card className="mb-2">
          <CardHeader>
            <CardTitle>Système</CardTitle>
          </CardHeader>
          <dl className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1 text-[13px]">
            <dt className="text-[var(--color-muted)]">OS</dt>
            <dd>
              {system.osName} {system.osVersion}
            </dd>
            <dt className="text-[var(--color-muted)]">Arch</dt>
            <dd>{system.arch}</dd>
            <dt className="text-[var(--color-muted)]">Uptime</dt>
            <dd>{formatUptime(system.uptimeSecs)}</dd>
          </dl>
        </Card>
      ) : null}
    </div>
  );
}

function SkeletonCard({ title }: { title: string }) {
  return (
    <Card className="opacity-70">
      <CardHeader>
        <CardTitle>{title}</CardTitle>
        <span className="text-sm text-[var(--color-muted)]">…</span>
      </CardHeader>
      <div className="h-3 w-full animate-pulse rounded-full bg-black/5" />
    </Card>
  );
}

function TempCell({ label, value }: { label: string; value: number | null }) {
  return (
    <div className="rounded-xl bg-black/[0.03] px-2 py-2">
      <div className="text-[11px] tracking-wide text-[var(--color-muted)] uppercase">
        {label}
      </div>
      <div className="mt-0.5 text-base font-semibold tabular-nums">
        {value != null ? `${value.toFixed(0)}°` : "—"}
      </div>
    </div>
  );
}
