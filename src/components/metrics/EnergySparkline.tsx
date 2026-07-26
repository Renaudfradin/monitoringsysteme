import type { EnergySample } from "@/types/metrics";

type EnergySparklineProps = {
  history: EnergySample[];
  label?: string;
  className?: string;
};

/** Lightweight SVG sparkline for energy samples (5 min or 24 h). */
export function EnergySparkline({
  history,
  label = "Historique de consommation",
  className,
}: EnergySparklineProps) {
  if (history.length < 2) {
    return (
      <div
        className={`flex h-14 items-center text-[12px] text-[var(--color-muted)] ${className ?? ""}`}
      >
        Historique en cours de collecte…
      </div>
    );
  }

  const watts = history.map((s) => s.watts);
  const min = Math.min(...watts);
  const max = Math.max(...watts);
  const span = Math.max(max - min, 0.5);
  const w = 320;
  const h = 56;
  const pad = 4;

  const points = watts
    .map((v, i) => {
      const x = pad + (i / (watts.length - 1)) * (w - pad * 2);
      const y = h - pad - ((v - min) / span) * (h - pad * 2);
      return `${x.toFixed(1)},${y.toFixed(1)}`;
    })
    .join(" ");

  const last = watts[watts.length - 1];
  const firstTs = history[0]?.ts;
  const lastTs = history[history.length - 1]?.ts;

  return (
    <div className={className}>
      <svg
        viewBox={`0 0 ${w} ${h}`}
        className="mt-3 h-14 w-full"
        role="img"
        aria-label={label}
      >
        <polyline
          fill="none"
          stroke="var(--color-accent)"
          strokeWidth="2"
          strokeLinejoin="round"
          strokeLinecap="round"
          points={points}
        />
      </svg>
      <div className="mt-1 flex justify-between text-[11px] text-[var(--color-muted)] tabular-nums">
        <span>
          {min.toFixed(1)}–{max.toFixed(1)} W
          {firstTs && lastTs
            ? ` · ${formatRange(firstTs, lastTs)}`
            : ""}
        </span>
        <span>{last.toFixed(1)} W</span>
      </div>
    </div>
  );
}

function formatRange(from: number, to: number): string {
  const spanH = (to - from) / 3600;
  if (spanH >= 2) return `${spanH.toFixed(0)} h`;
  const spanM = (to - from) / 60;
  if (spanM >= 2) return `${spanM.toFixed(0)} min`;
  return `${Math.max(1, to - from)} s`;
}
