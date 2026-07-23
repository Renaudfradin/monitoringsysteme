/** TypeScript mirrors of Rust metric models (camelCase serde). */

export interface CpuMetrics {
  usage: number;
  cores: number[];
  frequencyMhz: number;
  coreCount: number;
  loadAvg: number;
}

export interface MemoryMetrics {
  totalBytes: number;
  usedBytes: number;
  freeBytes: number;
  percent: number;
}

export interface DiskMetrics {
  totalBytes: number;
  usedBytes: number;
  freeBytes: number;
  percent: number;
  name: string;
}

export interface BatteryMetrics {
  level: number;
  state: string;
  timeRemainingSecs: number | null;
  charging: boolean;
}

export interface EnergySample {
  ts: number;
  watts: number;
}

export interface EnergyMetrics {
  watts: number;
  percent: number;
  history: EnergySample[];
  estimated: boolean;
}

export interface TemperatureMetrics {
  cpuCelsius: number | null;
  gpuCelsius: number | null;
  ssdCelsius: number | null;
  fansRpm: number[];
  available: boolean;
}

export interface SystemInfo {
  hostname: string;
  osName: string;
  osVersion: string;
  arch: string;
  uptimeSecs: number;
  model: string;
}

export interface AllMetrics {
  cpu: CpuMetrics | null;
  memory: MemoryMetrics | null;
  disk: DiskMetrics | null;
  energy: EnergyMetrics | null;
  battery: BatteryMetrics | null;
  temperature: TemperatureMetrics | null;
  system: SystemInfo | null;
}
