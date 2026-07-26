//! Short-lived energy ring buffer (5 min @ 1 Hz) + persistent 24 h history (1/min).

use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

use crate::models::{EnergyMetrics, EnergySample};
use crate::system::info::SystemInfoCache;

/// Maximum energy samples retained (~5 minutes at 1 Hz).
pub const ENERGY_HISTORY_CAPACITY: usize = 300;
/// Downsampled history: one average per minute for 24 hours.
pub const ENERGY_HISTORY_24H_CAPACITY: usize = 1_440;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct PersistedHistory {
    samples: Vec<EnergySample>,
}

pub struct EnergyHistory {
    samples: VecDeque<EnergySample>,
    samples_24h: VecDeque<EnergySample>,
    /// Current minute bucket (unix minute).
    bucket_minute: Option<u64>,
    bucket_sum: f64,
    bucket_count: u32,
    persist_path: Option<PathBuf>,
    dirty: bool,
    last_flush: Instant,
}

impl EnergyHistory {
    pub fn new() -> Self {
        Self {
            samples: VecDeque::with_capacity(ENERGY_HISTORY_CAPACITY),
            samples_24h: VecDeque::with_capacity(ENERGY_HISTORY_24H_CAPACITY),
            bucket_minute: None,
            bucket_sum: 0.0,
            bucket_count: 0,
            persist_path: None,
            dirty: false,
            last_flush: Instant::now(),
        }
    }

    pub fn set_persist_path(&mut self, path: PathBuf) {
        self.persist_path = Some(path);
    }

    pub fn load_from_disk(&mut self) {
        let Some(path) = self.persist_path.clone() else {
            return;
        };
        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(persisted) = serde_json::from_str::<PersistedHistory>(&data) {
                let cutoff = now_secs().saturating_sub(24 * 60 * 60);
                self.samples_24h.clear();
                for s in persisted.samples {
                    if s.ts >= cutoff {
                        self.samples_24h.push_back(s);
                    }
                }
                while self.samples_24h.len() > ENERGY_HISTORY_24H_CAPACITY {
                    self.samples_24h.pop_front();
                }
            }
        }
    }

    pub fn push(&mut self, watts: f64) {
        let ts = now_secs();
        if self.samples.len() >= ENERGY_HISTORY_CAPACITY {
            self.samples.pop_front();
        }
        self.samples.push_back(EnergySample { ts, watts });

        let minute = ts / 60;
        match self.bucket_minute {
            None => {
                self.bucket_minute = Some(minute);
                self.bucket_sum = watts;
                self.bucket_count = 1;
            }
            Some(prev) if prev == minute => {
                self.bucket_sum += watts;
                self.bucket_count += 1;
            }
            Some(prev) => {
                self.flush_bucket(prev);
                self.bucket_minute = Some(minute);
                self.bucket_sum = watts;
                self.bucket_count = 1;
            }
        }

        if self.dirty && self.last_flush.elapsed() >= Duration::from_secs(60) {
            self.save_to_disk();
        }
    }

    fn flush_bucket(&mut self, minute: u64) {
        if self.bucket_count == 0 {
            return;
        }
        let avg = self.bucket_sum / f64::from(self.bucket_count);
        let sample = EnergySample {
            ts: minute * 60,
            watts: (avg * 10.0).round() / 10.0,
        };
        if self.samples_24h.len() >= ENERGY_HISTORY_24H_CAPACITY {
            self.samples_24h.pop_front();
        }
        self.samples_24h.push_back(sample);
        self.dirty = true;
    }

    /// Flush the open minute bucket (e.g. on shutdown) then persist.
    pub fn flush_and_save(&mut self) {
        if let Some(minute) = self.bucket_minute {
            self.flush_bucket(minute);
            self.bucket_minute = None;
            self.bucket_sum = 0.0;
            self.bucket_count = 0;
        }
        self.save_to_disk();
    }

    pub fn save_to_disk(&mut self) {
        let Some(path) = self.persist_path.clone() else {
            return;
        };
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let persisted = PersistedHistory {
            samples: self.snapshot_24h(),
        };
        if let Ok(json) = serde_json::to_string_pretty(&persisted) {
            if fs::write(&path, json).is_ok() {
                self.dirty = false;
                self.last_flush = Instant::now();
            }
        }
    }

    pub fn snapshot(&self) -> Vec<EnergySample> {
        self.samples.iter().cloned().collect()
    }

    pub fn snapshot_24h(&self) -> Vec<EnergySample> {
        let cutoff = now_secs().saturating_sub(24 * 60 * 60);
        self.samples_24h
            .iter()
            .filter(|s| s.ts >= cutoff)
            .cloned()
            .collect()
    }

    #[allow(dead_code)]
    pub fn persist_path(&self) -> Option<&Path> {
        self.persist_path.as_deref()
    }
}

impl Default for EnergyHistory {
    fn default() -> Self {
        Self::new()
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Tiny TTL cache to avoid redundant heavy refreshes within the same tick.
#[allow(dead_code)]
pub struct TtlCache<T: Clone> {
    value: Option<(Instant, T)>,
    ttl: Duration,
}

#[allow(dead_code)]
impl<T: Clone> TtlCache<T> {
    pub fn new(ttl: Duration) -> Self {
        Self { value: None, ttl }
    }

    pub fn get_or_insert_with<F>(&mut self, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        if let Some((at, ref v)) = self.value {
            if at.elapsed() < self.ttl {
                return v.clone();
            }
        }
        let v = f();
        self.value = Some((Instant::now(), v.clone()));
        v
    }
}

/// Debounce state for high CPU / RAM notifications.
pub struct AlertState {
    pub last_cpu_alert: Mutex<Option<Instant>>,
    pub last_ram_alert: Mutex<Option<Instant>>,
}

impl AlertState {
    pub fn new() -> Self {
        Self {
            last_cpu_alert: Mutex::new(None),
            last_ram_alert: Mutex::new(None),
        }
    }

    /// Returns true if an alert should fire (threshold crossed + cooldown elapsed).
    pub fn should_alert_cpu(&self, cooldown: Duration) -> bool {
        let mut last = self.last_cpu_alert.lock();
        match *last {
            Some(at) if at.elapsed() < cooldown => false,
            _ => {
                *last = Some(Instant::now());
                true
            }
        }
    }

    pub fn should_alert_ram(&self, cooldown: Duration) -> bool {
        let mut last = self.last_ram_alert.lock();
        match *last {
            Some(at) if at.elapsed() < cooldown => false,
            _ => {
                *last = Some(Instant::now());
                true
            }
        }
    }
}

impl Default for AlertState {
    fn default() -> Self {
        Self::new()
    }
}

/// Shared runtime state for the active provider.
pub struct ProviderState {
    pub energy_history: Mutex<EnergyHistory>,
    pub last_energy: Mutex<Option<EnergyMetrics>>,
    pub last_cpu: Mutex<Option<f32>>,
    pub last_ram: Mutex<Option<f32>>,
    pub alerts: AlertState,
    /// Cached OS / hardware inventory (expensive on macOS).
    pub system_info: SystemInfoCache,
}

impl ProviderState {
    pub fn new() -> Self {
        Self {
            energy_history: Mutex::new(EnergyHistory::new()),
            last_energy: Mutex::new(None),
            last_cpu: Mutex::new(None),
            last_ram: Mutex::new(None),
            alerts: AlertState::new(),
            system_info: SystemInfoCache::new(),
        }
    }
}

impl Default for ProviderState {
    fn default() -> Self {
        Self::new()
    }
}
