import type { EnergySample } from "@/types/metrics";

type EnergySparklineProps = {
  history: EnergySample[];
  className?: string;
};

/** Lightweight SVG sparkline for the last ~5 minutes of power samples. */
export function EnergySparkline({ history, className }: EnergySparklineProps) {
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

  return (
    <svg
      viewBox={`0 0 ${w} ${h}`}
      className={`mt-3 h-14 w-full ${className ?? ""}`}
      role="img"
      aria-label="Historique de consommation sur 5 minutes"
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
  );
}
