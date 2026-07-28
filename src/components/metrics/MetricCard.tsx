import type { ReactNode } from "react";
import { Card, CardHeader, CardTitle } from "@/components/ui/card";
import { MetricProgress } from "@/components/ui/progress";
import { cn } from "@/lib/utils";

type MetricCardProps = {
  title: string;
  percent: number;
  detail?: string;
  footer?: ReactNode;
  className?: string;
  children?: ReactNode;
  invertTone?: boolean;
};

export function MetricCard({
  title,
  percent,
  detail,
  footer,
  className,
  children,
  invertTone = false,
}: MetricCardProps) {
  const display = Math.round(Math.max(0, Math.min(100, percent)));
  return (
    <Card className={cn("animate-in fade-in duration-300", className)}>
      <CardHeader>
        <CardTitle>{title}</CardTitle>
        <div className="text-right">
          <span className="text-2xl font-semibold tracking-tight tabular-nums">
            {display}
          </span>
          <span className="ml-1 text-sm text-[var(--color-muted)]">%</span>
        </div>
      </CardHeader>
      <MetricProgress value={percent} invertTone={invertTone} />
      {detail ? (
        <p className="mt-2 text-[13px] text-[var(--color-muted)]">{detail}</p>
      ) : null}
      {children}
      {footer ? <div className="mt-3">{footer}</div> : null}
    </Card>
  );
}
