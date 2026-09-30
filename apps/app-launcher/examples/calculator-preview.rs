//! Native RGB565 screenshots of the actual firmware calculator widgets.
//! cargo run -p app-launcher --features screenshots --example calculator-preview -- artifacts/calculator-mac
use app_launcher::headless::HeadlessBackend;
use app_launcher::{build_launcher_ui, ActiveApp, LauncherState};
use calculator::{BinaryOp, CalcMode};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tiny_flutter::{App, Size};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = PathBuf::from(
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| "artifacts/calculator-mac".into()),
    );
    std::fs::create_dir_all(&out)?;
    for (mode, name) in [
        (CalcMode::Basic, "basic"),
        (CalcMode::Scientific, "scientific"),
        (CalcMode::Programmer, "programmer"),
    ] {
        let state = Arc::new(Mutex::new(LauncherState::new()));
        state.lock().unwrap().settings.light_appearance =
            std::env::args().nth(2).as_deref() == Some("light");
        state.lock().unwrap().open_app("calculator");
        if let ActiveApp::Calculator(calc) = &state.lock().unwrap().active_app {
            let mut c = calc.lock().unwrap();
            c.set_mode(mode);
            match mode {
                CalcMode::Basic => {
                    for d in "1234".chars() {
                        c.input_digit(d);
                    }
                    c.binary(BinaryOp::Multiply);
                    c.input_digit('5');
                    c.calculate();
                }
                CalcMode::Scientific => {
                    c.open_parenthesis();
                    c.input_digit('3');
                    c.binary(BinaryOp::Add);
                    c.input_digit('2');
                    c.close_parenthesis();
                    c.binary(BinaryOp::Power);
                    c.input_digit('3');
                    c.calculate();
                }
                CalcMode::Programmer => c.programmer.set_value(0x0123456789ABCDEF),
            }
        }
        let size = Size::new(1024.0, 600.0);
        let mut backend = HeadlessBackend::new(1024, 600);
        App::new(build_launcher_ui(state, size), size).step(&mut backend);
        #[cfg(feature = "screenshots")]
        backend.screenshot(&out.join(format!("{name}.png")))?;
    }
    Ok(())
}
