import { invoke } from "@tauri-apps/api/core";
import type {
  AppSettings,
  BatteryMetrics,
  CpuMetrics,
  DiskMetrics,
  EnergyMetrics,
  EnergySample,
  GpuLiveMetrics,
  MemoryMetrics,
  NetworkMetrics,
  PluginInfo,
  ProcessMetrics,
  SystemInfo,
  TemperatureMetrics,
} from "@/types/metrics";

/** Typed wrappers around independent Tauri metric commands. */

export const getCpu = () => invoke<CpuMetrics>("get_cpu");
export const getMemory = () => invoke<MemoryMetrics>("get_memory");
export const getDisk = () => invoke<DiskMetrics>("get_disk");
export const getEnergy = () => invoke<EnergyMetrics>("get_energy");
export const getBattery = () => invoke<BatteryMetrics | null>("get_battery");
export const getTemperature = () => invoke<TemperatureMetrics>("get_temperature");
export const getSystem = () => invoke<SystemInfo>("get_system");
export const getProcesses = () => invoke<ProcessMetrics>("get_processes");
export const getNetwork = () => invoke<NetworkMetrics>("get_network");
export const getGpu = () => invoke<GpuLiveMetrics>("get_gpu");
export const getEnergyHistory24h = () =>
  invoke<EnergySample[]>("get_energy_history_24h");
export const exportMetrics = (format: "json" | "csv", path: string) =>
  invoke<void>("export_metrics", { format, path });
export const checkResourceAlerts = () =>
  invoke<string[]>("check_resource_alerts");
export const getSettings = () => invoke<AppSettings>("get_settings");
export const setSettings = (settings: AppSettings) =>
  invoke<AppSettings>("set_settings", { settings });
export const listPlugins = () => invoke<PluginInfo[]>("list_plugins");
export const invokePlugin = (id: string) =>
  invoke<unknown>("invoke_plugin", { id });
