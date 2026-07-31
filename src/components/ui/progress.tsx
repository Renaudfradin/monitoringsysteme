import * as ProgressPrimitive from "@radix-ui/react-progress";
import { cn, metricColorClass } from "@/lib/utils";

type MetricProgressProps = {
  value: number;
  className?: string;
  invertTone?: boolean;
};

/** Wide progress bar with dynamic green / orange / red fill. */
export function MetricProgress({
  value,
  className,
  invertTone = false,
}: MetricProgressProps) {
  const clamped = Math.max(0, Math.min(100, value));
  return (
    <ProgressPrimitive.Root
      value={clamped}
      className={cn(
        "relative h-3 w-full overflow-hidden rounded-full bg-black/5",
        className,
      )}
    >
      <ProgressPrimitive.Indicator
        className={cn(
          "h-full rounded-full transition-[width,background-color] duration-500 ease-out",
          metricColorClass(clamped, invertTone),
        )}
        style={{ width: `${clamped}%` }}
      />
    </ProgressPrimitive.Root>
  );
}
