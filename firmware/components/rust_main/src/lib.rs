//! Rust Pad runtime over the shared ESP-IDF C HAL. Host tests use the same runtime and UI code.
#[cfg(any(target_os = "espidf", test))]
mod cadence;
#[cfg(target_os = "espidf")]
mod ffi;
mod persistence;
mod diagnostics;
pub mod usage;
pub mod runtime;
