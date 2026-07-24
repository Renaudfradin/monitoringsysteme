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
import type {
  CpuInfo,
  DisplayInfo,
  GpuInfo,
  MemoryInfo,
  StorageInfo,
  SystemInfo,
} from "@/types/metrics";

export function Dashboard() {
  const { metrics, error, loading } = useMetrics();
  const { cpu, memory, disk, energy, battery, temperature, system } = metrics;

  const machineLabel = system
    ? [system.modelName, system.model].filter(Boolean).join(" · ") || system.model
    : null;

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
            {machineLabel} · {system.hostname}
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
          detail={`${cpu.coreCount} cœurs · ${cpu.frequencyMhz} MHz · load ${cpu.loadAvg.toFixed(2)}${
            system?.cpu?.brand ? ` · ${system.cpu.brand}` : ""
          }`}
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
          detail={`${formatBytes(memory.usedBytes)} utilisés · ${formatBytes(memory.totalBytes)} total${
            system?.memory?.typeName ? ` · ${system.memory.typeName}` : ""
          }`}
        />
      ) : (
        <SkeletonCard title="RAM" />
      )}

      {disk ? (
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

      {system ? <SystemDetails system={system} /> : null}
    </div>
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
          <InfoRow label="OS" value={system.osLongName ?? `${system.osName} ${system.osVersion}`} />
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
      {system.gpu.map((gpu, i) => (
        <GpuDetails key={`${gpu.name}-${i}`} gpu={gpu} index={i} />
      ))}
      {system.storage.map((disk, i) => (
        <StorageDetails key={`${disk.bsdName ?? disk.name}-${i}`} disk={disk} />
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
