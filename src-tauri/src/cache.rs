//! Short-lived metric cache and circular energy history (5 minutes @ 1 Hz).

use std::collections::VecDeque;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use parking_lot::Mutex;

use crate::models::{EnergyMetrics, EnergySample};
use crate::system::info::SystemInfoCache;

/// Maximum energy samples retained (~5 minutes at 1 Hz).
pub const ENERGY_HISTORY_CAPACITY: usize = 300;

pub struct EnergyHistory {
    samples: VecDeque<EnergySample>,
}

impl EnergyHistory {
    pub fn new() -> Self {
        Self {
            samples: VecDeque::with_capacity(ENERGY_HISTORY_CAPACITY),
        }
    }

    pub fn push(&mut self, watts: f64) {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        if self.samples.len() >= ENERGY_HISTORY_CAPACITY {
            self.samples.pop_front();
        }
        self.samples.push_back(EnergySample { ts, watts });
    }

    pub fn snapshot(&self) -> Vec<EnergySample> {
        self.samples.iter().cloned().collect()
    }
}

impl Default for EnergyHistory {
    fn default() -> Self {
        Self::new()
    }
}

/// Tiny TTL cache to avoid redundant heavy refreshes within the same tick.
/// Reserved for future per-metric caching across commands.
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

/// Shared runtime state for the active provider.
pub struct ProviderState {
    pub energy_history: Mutex<EnergyHistory>,
    pub last_energy: Mutex<Option<EnergyMetrics>>,
    /// Cached OS / hardware inventory (expensive on macOS).
    pub system_info: SystemInfoCache,
}

impl ProviderState {
    pub fn new() -> Self {
        Self {
            energy_history: Mutex::new(EnergyHistory::new()),
            last_energy: Mutex::new(None),
            system_info: SystemInfoCache::new(),
        }
    }
}

impl Default for ProviderState {
    fn default() -> Self {
        Self::new()
    }
}
