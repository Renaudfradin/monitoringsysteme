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
  history24h: EnergySample[];
  estimated: boolean;
}

export interface TempSensor {
  label: string;
  celsius: number;
}

export interface TemperatureMetrics {
  cpuCelsius: number | null;
  gpuCelsius: number | null;
  ssdCelsius: number | null;
  batteryCelsius: number | null;
  maxCelsius: number | null;
  fansRpm: number[];
  sensors: TempSensor[];
  available: boolean;
}

export interface ProcessEntry {
  pid: number;
  name: string;
  cpuPercent: number;
  memoryBytes: number;
}

export interface ProcessMetrics {
  topCpu: ProcessEntry[];
  topMemory: ProcessEntry[];
}

export interface NetworkInterface {
  name: string;
  rxBytesPerSec: number;
  txBytesPerSec: number;
  rxTotalBytes: number;
  txTotalBytes: number;
}

export interface NetworkMetrics {
  interfaces: NetworkInterface[];
  totalRxBytesPerSec: number;
  totalTxBytesPerSec: number;
}

export interface FanInfo {
  name: string;
  rpm: number;
}

export interface GpuLiveMetrics {
  name: string;
  utilizationPercent: number | null;
  temperatureCelsius: number | null;
  fans: FanInfo[];
  available: boolean;
}

export type SectionId =
  | "cpu"
  | "memory"
  | "disk"
  | "energy"
  | "battery"
  | "temperature"
  | "gpu"
  | "network"
  | "processes"
  | "export"
  | "plugins"
  | "system";

export type VisibleSections = Record<SectionId, boolean>;

export type TrayMetricId = "cpu" | "memory" | "energy";

export interface TrayDisplay {
  /** Affiche le texte à côté de l'icône dans la barre de menu. */
  enabled: boolean;
  cpu: boolean;
  memory: boolean;
  energy: boolean;
}

export interface AppSettings {
  theme: "system" | "light" | "dark" | string;
  notificationsEnabled: boolean;
  cpuAlertThreshold: number;
  ramAlertThreshold: number;
  launchAtLogin: boolean;
  visibleSections: VisibleSections;
  trayDisplay: TrayDisplay;
}

export interface PluginInfo {
  id: string;
  name: string;
  description: string;
  builtin: boolean;
}

export interface CpuInfo {
  brand: string;
  vendor: string | null;
  cores: number | null;
  performanceCores: number | null;
  efficiencyCores: number | null;
  frequencyMhz: number | null;
}

export interface GpuInfo {
  name: string;
  chipset: string | null;
  vendor: string | null;
  cores: number | null;
  vramBytes: number | null;
  metalSupport: string | null;
  bus: string | null;
}

export interface MemoryModule {
  sizeBytes: number | null;
  typeName: string | null;
  speedMhz: number | null;
  manufacturer: string | null;
  partNumber: string | null;
  serial: string | null;
  slot: string | null;
}

export interface MemoryInfo {
  totalBytes: number | null;
  typeName: string | null;
  manufacturer: string | null;
  modules: MemoryModule[];
}

export interface StorageInfo {
  name: string;
  model: string | null;
  mediumType: string | null;
  protocol: string | null;
  sizeBytes: number | null;
  serial: string | null;
  smartStatus: string | null;
  mountPoint: string | null;
  bsdName: string | null;
}

export interface DisplayInfo {
  name: string;
  resolution: string | null;
  pixelResolution: string | null;
  displayType: string | null;
  connection: string | null;
  vendorId: string | null;
  productId: string | null;
  serial: string | null;
  main: boolean;
}

export interface SystemInfo {
  hostname: string;
  model: string;
  modelName: string | null;
  modelNumber: string | null;
  serialNumber: string | null;
  hardwareUuid: string | null;
  firmwareVersion: string | null;
  osName: string;
  osVersion: string;
  osBuild: string | null;
  osLongName: string | null;
  kernelVersion: string | null;
  arch: string;
  uptimeSecs: number;
  cpu: CpuInfo | null;
  gpu: GpuInfo[];
  memory: MemoryInfo | null;
  storage: StorageInfo[];
  displays: DisplayInfo[];
}

export interface AllMetrics {
  cpu: CpuMetrics | null;
  memory: MemoryMetrics | null;
  disk: DiskMetrics | null;
  energy: EnergyMetrics | null;
  battery: BatteryMetrics | null;
  temperature: TemperatureMetrics | null;
  processes: ProcessMetrics | null;
  network: NetworkMetrics | null;
  gpu: GpuLiveMetrics | null;
  system: SystemInfo | null;
}
