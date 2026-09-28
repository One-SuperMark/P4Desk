use calculator::{build_calculator_ui, CalcState};
use std::sync::{Arc, Mutex};
use tiny_flutter::prelude::*;

fn main() {
    println!("[Calculator] Desktop Simulator Starting...");

    let screen_w = 480;
    let screen_h = 480;
    let state = Arc::new(Mutex::new(CalcState::new()));
    let state_clone = state.clone();
    if let Err(e) = run_simulator(
        "TinyFlutter AMOLED Calculator",
        screen_w,
        screen_h,
        move |current_size| build_calculator_ui(state_clone.clone(), current_size),
    ) {
        eprintln!("[Simulator] Simulator exited: {}", e);
    } else {
        println!("[Simulator] Window closed.");
    }
}
