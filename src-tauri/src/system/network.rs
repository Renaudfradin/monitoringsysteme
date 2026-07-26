//! Network interface throughput (bytes/s deltas via sysinfo).

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

use sysinfo::Networks;

use crate::error::MetricError;
use crate::models::{NetworkInterface, NetworkMetrics};

struct NetSnapshot {
    at: Instant,
    // name -> (rx_total, tx_total)
    totals: HashMap<String, (u64, u64)>,
}

static LAST: Mutex<Option<NetSnapshot>> = Mutex::new(None);

pub fn collect() -> Result<NetworkMetrics, MetricError> {
    let mut networks = Networks::new_with_refreshed_list();
    networks.refresh(true);

    let mut current: HashMap<String, (u64, u64)> = HashMap::new();
    for (name, data) in networks.iter() {
        current.insert(
            name.to_string(),
            (data.total_received(), data.total_transmitted()),
        );
    }

    let now = Instant::now();
    let mut interfaces = Vec::new();
    let mut total_rx = 0.0_f64;
    let mut total_tx = 0.0_f64;

    {
        let mut guard = LAST.lock().unwrap_or_else(|e| e.into_inner());
        let elapsed = guard
            .as_ref()
            .map(|s| now.duration_since(s.at).as_secs_f64().max(0.001))
            .unwrap_or(1.0);

        for (name, &(rx, tx)) in &current {
            let (rx_rate, tx_rate) = if let Some(prev) = guard.as_ref().and_then(|s| s.totals.get(name))
            {
                let drx = rx.saturating_sub(prev.0) as f64 / elapsed;
                let dtx = tx.saturating_sub(prev.1) as f64 / elapsed;
                (drx, dtx)
            } else {
                (0.0, 0.0)
            };
            total_rx += rx_rate;
            total_tx += tx_rate;
            interfaces.push(NetworkInterface {
                name: name.clone(),
                rx_bytes_per_sec: rx_rate,
                tx_bytes_per_sec: tx_rate,
                rx_total_bytes: rx,
                tx_total_bytes: tx,
            });
        }

        *guard = Some(NetSnapshot {
            at: now,
            totals: current,
        });
    }

    interfaces.sort_by(|a, b| {
        (b.rx_bytes_per_sec + b.tx_bytes_per_sec)
            .partial_cmp(&(a.rx_bytes_per_sec + a.tx_bytes_per_sec))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(NetworkMetrics {
        interfaces,
        total_rx_bytes_per_sec: total_rx,
        total_tx_bytes_per_sec: total_tx,
    })
}
