use calculator::{build_calculator_ui, build_mode_selector, CalcState};
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
        move |current_size| {
            let width = (current_size.width - 32.0).clamp(1.0, 340.0);
            Stack::new()
                .push(build_calculator_ui(state_clone.clone(), current_size))
                .push(
                    Positioned::new(build_mode_selector(
                        state_clone.clone(),
                        Size::new(width, 34.0),
                    ))
                    .left(current_size.width - width)
                    .top(0.0),
                )
        },
    ) {
        eprintln!("[Simulator] Simulator exited: {}", e);
    } else {
        println!("[Simulator] Window closed.");
    }
}
