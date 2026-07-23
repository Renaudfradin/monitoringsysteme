import { invoke } from "@tauri-apps/api/core";
import type {
  BatteryMetrics,
  CpuMetrics,
  DiskMetrics,
  EnergyMetrics,
  MemoryMetrics,
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
