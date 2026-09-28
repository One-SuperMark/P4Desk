pub mod backend;
pub mod esp_wasm;
pub mod simulator;

pub use backend::{PlatformBackend, RawTouchFrame, RawTouchPoint};
pub use esp_wasm::EspWasmBackend;
#[cfg(feature = "simulator")]
pub use simulator::{run_simulator, run_simulator_animated, SimulatorBackend};
