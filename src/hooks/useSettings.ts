import { useCallback, useEffect, useState } from "react";
import { enable, disable, isEnabled } from "@tauri-apps/plugin-autostart";
import { getSettings, setSettings as saveSettings } from "@/services/tauri";
import type { AppSettings, TrayDisplay, VisibleSections } from "@/types/metrics";

export const defaultVisibleSections: VisibleSections = {
  cpu: true,
  memory: true,
  disk: true,
  energy: true,
  battery: true,
  temperature: true,
  gpu: true,
  network: true,
  processes: true,
  export: true,
  plugins: true,
  system: true,
};

export const defaultTrayDisplay: TrayDisplay = {
  enabled: true,
  cpu: true,
  memory: true,
  energy: true,
};

const defaults: AppSettings = {
  theme: "system",
  notificationsEnabled: true,
  cpuAlertThreshold: 90,
  ramAlertThreshold: 90,
  launchAtLogin: false,
  visibleSections: defaultVisibleSections,
  trayDisplay: defaultTrayDisplay,
};

function applyTheme(theme: string) {
  const root = document.documentElement;
  const preferDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
  const dark = theme === "dark" || (theme === "system" && preferDark);
  root.dataset.theme = dark ? "dark" : "light";
}

export function useSettings() {
  const [settings, setSettingsState] = useState<AppSettings>(defaults);
  const [ready, setReady] = useState(false);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const s = await getSettings();
        const normalized: AppSettings = {
          ...defaults,
          ...s,
          visibleSections: {
            ...defaultVisibleSections,
            ...(s.visibleSections ?? {}),
          },
          trayDisplay: {
            ...defaultTrayDisplay,
            ...(s.trayDisplay ?? {}),
          },
        };
        if (!cancelled) {
          setSettingsState(normalized);
          applyTheme(normalized.theme);
        }
        const auto = await isEnabled().catch(() => false);
        if (!cancelled && auto !== normalized.launchAtLogin) {
          const next = { ...normalized, launchAtLogin: auto };
          setSettingsState(next);
        }
      } catch {
        applyTheme("system");
      } finally {
        if (!cancelled) setReady(true);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    if (!ready) return;
    applyTheme(settings.theme);
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    const onChange = () => applyTheme(settings.theme);
    mq.addEventListener("change", onChange);
    return () => mq.removeEventListener("change", onChange);
  }, [settings.theme, ready]);

  const update = useCallback(async (patch: Partial<AppSettings>) => {
    setSettingsState((prev) => {
      const next: AppSettings = {
        ...prev,
        ...patch,
        visibleSections: patch.visibleSections
          ? { ...prev.visibleSections, ...patch.visibleSections }
          : prev.visibleSections,
        trayDisplay: patch.trayDisplay
          ? { ...prev.trayDisplay, ...patch.trayDisplay }
          : prev.trayDisplay,
      };
      void (async () => {
        try {
          const saved = await saveSettings(next);
          setSettingsState({
            ...defaults,
            ...saved,
            visibleSections: {
              ...defaultVisibleSections,
              ...(saved.visibleSections ?? {}),
            },
            trayDisplay: {
              ...defaultTrayDisplay,
              ...(saved.trayDisplay ?? {}),
            },
          });
          if (patch.launchAtLogin != null) {
            if (patch.launchAtLogin) await enable();
            else await disable();
          }
        } catch {
          /* keep optimistic UI */
        }
      })();
      return next;
    });
  }, []);

  return { settings, update, ready };
}
