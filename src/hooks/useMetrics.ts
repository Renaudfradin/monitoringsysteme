import { useCallback, useEffect, useRef, useState } from "react";
import {
  checkResourceAlerts,
  getBattery,
  getCpu,
  getDisk,
  getEnergy,
  getGpu,
  getMemory,
  getNetwork,
  getProcesses,
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
  processes: null,
  network: null,
  gpu: null,
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
  const alertTick = useRef(0);

  const refresh = useCallback(async () => {
    if (document.visibilityState === "hidden") return;
    if (inFlight.current) return;
    inFlight.current = true;
    try {
      const [
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
      ] = await Promise.all([
        getCpu().catch(() => null),
        getMemory().catch(() => null),
        getDisk().catch(() => null),
        getEnergy().catch(() => null),
        getBattery().catch(() => null),
        getTemperature().catch(() => null),
        getProcesses().catch(() => null),
        getNetwork().catch(() => null),
        getGpu().catch(() => null),
        getSystem().catch(() => null),
      ]);

      setMetrics({
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
      });
      setError(null);

      // Check alerts every ~15 s to avoid spam while keeping responsiveness.
      alertTick.current += 1;
      if (alertTick.current % 15 === 0) {
        void checkResourceAlerts().catch(() => undefined);
      }
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
