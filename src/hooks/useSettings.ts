import { useCallback, useEffect, useState } from "react";
import { enable, disable, isEnabled } from "@tauri-apps/plugin-autostart";
import { getSettings, setSettings as saveSettings } from "@/services/tauri";
import type { AppSettings } from "@/types/metrics";

const defaults: AppSettings = {
  theme: "system",
  notificationsEnabled: true,
  cpuAlertThreshold: 90,
  ramAlertThreshold: 90,
  launchAtLogin: false,
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
        if (!cancelled) {
          setSettingsState(s);
          applyTheme(s.theme);
        }
        const auto = await isEnabled().catch(() => false);
        if (!cancelled && auto !== s.launchAtLogin) {
          const next = { ...s, launchAtLogin: auto };
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
      const next = { ...prev, ...patch };
      void (async () => {
        try {
          const saved = await saveSettings(next);
          setSettingsState(saved);
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
