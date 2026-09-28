//! Rust Pad runtime over the shared ESP-IDF C HAL. Host tests use the same runtime and UI code.
#[cfg(target_os = "espidf")]
mod ffi;
pub mod runtime;
