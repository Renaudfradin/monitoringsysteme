//! Battery metrics via the `battery` crate.

use battery::Manager;
use battery::State;

use crate::error::MetricError;
use crate::models::BatteryMetrics;

pub fn collect() -> Result<Option<BatteryMetrics>, MetricError> {
    let manager = Manager::new().map_err(|e| MetricError::Internal(e.to_string()))?;
    let mut batteries = manager
        .batteries()
        .map_err(|e| MetricError::Internal(e.to_string()))?;

    let Some(bat) = batteries.next() else {
        return Ok(None);
    };
    let bat = bat.map_err(|e| MetricError::Internal(e.to_string()))?;

    let level = bat
        .state_of_charge()
        .get::<battery::units::ratio::percent>();
    let state = match bat.state() {
        State::Charging => "charging",
        State::Discharging => "discharging",
        State::Full => "full",
        State::Empty => "empty",
        State::Unknown => "unknown",
        _ => "unknown",
    }
    .to_string();

    let charging = matches!(bat.state(), State::Charging);
    let time_remaining_secs = if charging {
        bat.time_to_full()
            .map(|t| t.get::<battery::units::time::second>() as u64)
    } else {
        bat.time_to_empty()
            .map(|t| t.get::<battery::units::time::second>() as u64)
    };

    Ok(Some(BatteryMetrics {
        level,
        state,
        time_remaining_secs,
        charging,
    }))
}
