import { useCallback, useEffect, useRef, useState } from "react";
import {
  getBattery,
  getCpu,
  getDisk,
  getEnergy,
  getMemory,
  getSystem,
  getTemperature,
} from "@/services/tauri";
import type { AllMetrics } from "@/types/metrics";

const POLL_MS = 1000;

const empty: AllMetrics = {
  cpu: null,
  memory: null,
  disk: null,
  energy: null,
  battery: null,
  temperature: null,
  system: null,
};

/**
 * Polls all metric commands every second while the document is visible.
 * Failures for individual metrics are swallowed so the dashboard stays resilient.
 */
export function useMetrics() {
  const [metrics, setMetrics] = useState<AllMetrics>(empty);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const inFlight = useRef(false);

  const refresh = useCallback(async () => {
    if (document.visibilityState === "hidden") return;
    if (inFlight.current) return;
    inFlight.current = true;
    try {
      const [cpu, memory, disk, energy, battery, temperature, system] =
        await Promise.all([
          getCpu().catch(() => null),
          getMemory().catch(() => null),
          getDisk().catch(() => null),
          getEnergy().catch(() => null),
          getBattery().catch(() => null),
          getTemperature().catch(() => null),
          getSystem().catch(() => null),
        ]);

      setMetrics({
        cpu,
        memory,
        disk,
        energy,
        battery,
        temperature,
        system,
      });
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
      inFlight.current = false;
    }
  }, []);

  useEffect(() => {
    void refresh();
    const id = window.setInterval(() => void refresh(), POLL_MS);
    const onVis = () => {
      if (document.visibilityState === "visible") void refresh();
    };
    document.addEventListener("visibilitychange", onVis);
    return () => {
      window.clearInterval(id);
      document.removeEventListener("visibilitychange", onVis);
    };
  }, [refresh]);

  return { metrics, error, loading };
}
