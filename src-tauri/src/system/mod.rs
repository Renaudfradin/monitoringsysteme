//! Independent metric collectors. Each module is platform-agnostic where possible;
//! platform-specific details live in `provider::*`.

pub mod battery;
pub mod cpu;
pub mod disk;
pub mod energy;
pub mod gpu;
pub mod info;
pub mod memory;
pub mod network;
pub mod processes;
pub mod temperature;
