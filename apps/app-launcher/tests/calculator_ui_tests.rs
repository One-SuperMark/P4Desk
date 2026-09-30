use app_launcher::headless::HeadlessBackend;
use app_launcher::{build_launcher_ui, ActiveApp, LauncherState};
use calculator::{
    calculator_layout, mode_selector_layout, BinaryOp as B, CalcMode, CalcState, KeyAction as A,
    UnaryOp as U,
};
use std::sync::{Arc, Mutex};
use tiny_flutter::prelude::*;

struct Harness {
    launcher: Arc<Mutex<LauncherState>>,
    calc: Arc<Mutex<CalcState>>,
    app: App,
    backend: HeadlessBackend,
}
impl Harness {
    fn new() -> Self {
        let launcher = Arc::new(Mutex::new(LauncherState::new()));
        launcher.lock().unwrap().open_app("calculator");
        let calc = match &launcher.lock().unwrap().active_app {
            ActiveApp::Calculator(c) => c.clone(),
            _ => unreachable!(),
        };
        let size = Size::new(1024.0, 600.0);
        let mut backend = HeadlessBackend::new(1024, 600);
        let mut app = App::new(build_launcher_ui(launcher.clone(), size), size);
        app.step(&mut backend);
        Self {
            launcher,
            calc,
            app,
            backend,
        }
    }
    fn touch(&mut self, event: TouchEvent) {
        self.backend.event(event);
        self.app.step_with_builder(&mut self.backend, |size| {
            build_launcher_ui(self.launcher.clone(), size)
        });
    }
    fn key_point(&self, action: A) -> Point {
        if matches!(action, A::Mode(_)) {
            let bounds = app_launcher::launcher_ui::calculator_mode_selector_bounds(1024.0);
            let keys = mode_selector_layout(
                &self.calc.lock().unwrap(),
                Size::new(bounds.width, bounds.height),
            );
            let key = keys.iter().find(|key| key.action == action).unwrap();
            return Point::new(
                bounds.x + key.rect.x + key.rect.width * 0.5,
                bounds.y + key.rect.y + key.rect.height * 0.5,
            );
        }
        let layout = calculator_layout(&self.calc.lock().unwrap(), Size::new(976.0, 506.0));
        let r = layout.key(action).expect("key exists").rect;
        Point::new(r.x + r.width * 0.5 + 24.0, r.y + r.height * 0.5 + 78.0)
    }
    fn tap(&mut self, action: A) {
        let point = self.key_point(action);
        self.touch(TouchEvent::Down(point));
        self.touch(TouchEvent::Up(point));
    }
}
#[test]
fn touch_basic_scientific_and_programmer_calculate_real_values() {
    let mut h = Harness::new();
    for a in [A::Digit('7'), A::Binary(B::Add), A::Digit('5'), A::Equals] {
        h.tap(a);
    }
    assert_eq!(h.calc.lock().unwrap().value(), Ok(12.0));
    h.tap(A::Mode(CalcMode::Scientific));
    h.tap(A::Clear);
    h.tap(A::Digit('3'));
    h.tap(A::Digit('0'));
    h.tap(A::Unary(U::Sin));
    assert!((h.calc.lock().unwrap().value().unwrap() - 0.5).abs() < 1e-12);
    h.tap(A::Mode(CalcMode::Programmer));
    h.tap(A::Digit('F'));
    h.tap(A::Binary(B::And));
    h.tap(A::Digit('3'));
    h.tap(A::Equals);
    assert_eq!(h.calc.lock().unwrap().programmer.value, 3);
}
#[test]
fn programmer_base_disables_invalid_keys_and_bit_taps_change_value() {
    let mut h = Harness::new();
    h.tap(A::Mode(CalcMode::Programmer));
    h.tap(A::Radix(8));
    h.tap(A::Digit('9'));
    assert_eq!(h.calc.lock().unwrap().programmer.value, 0);
    h.tap(A::Bit(63));
    h.tap(A::Bit(0));
    assert_eq!(h.calc.lock().unwrap().programmer.value, 0x8000000000000001);
    h.tap(A::ToggleBinary);
    assert!(!h.calc.lock().unwrap().programmer.show_binary);
    h.tap(A::Character(true));
    assert!(h.calc.lock().unwrap().programmer.unicode);
}
#[test]
fn touch_cancel_and_drag_do_not_input_a_digit() {
    let mut h = Harness::new();
    let point = h.key_point(A::Digit('7'));
    h.touch(TouchEvent::Down(point));
    h.touch(TouchEvent::Cancel);
    h.touch(TouchEvent::Up(point));
    assert_eq!(h.calc.lock().unwrap().value(), Ok(0.0));
    h.touch(TouchEvent::Down(point));
    h.touch(TouchEvent::Move(Point::new(1.0, 1.0)));
    h.touch(TouchEvent::Up(Point::new(1.0, 1.0)));
    assert_eq!(h.calc.lock().unwrap().value(), Ok(0.0));
}
#[test]
fn every_keyboard_target_stays_on_screen_and_does_not_overlap() {
    for mode in [CalcMode::Basic, CalcMode::Scientific, CalcMode::Programmer] {
        let mut s = CalcState::new();
        s.set_mode(mode);
        let layout = calculator_layout(&s, Size::new(976.0, 506.0));
        let keyboard: Vec<_> = layout
            .keys
            .iter()
            .filter(|k| k.rect.y >= layout.keyboard.y)
            .collect();
        assert_eq!(
            keyboard.len(),
            match mode {
                CalcMode::Basic => 20,
                CalcMode::Scientific => 50,
                CalcMode::Programmer => 42,
            }
        );
        for (i, k) in keyboard.iter().enumerate() {
            assert!(
                k.rect.x >= 0.0
                    && k.rect.y >= 0.0
                    && k.rect.right() <= 976.01
                    && k.rect.bottom() <= 506.01
            );
            assert!(k.rect.height >= 40.0);
            for other in &keyboard[i + 1..] {
                assert!(k.rect.intersect(&other.rect).is_none());
            }
        }
    }
}
#[test]
fn home_preserves_all_mode_state_and_close_resets() {
    let mut h = Harness::new();
    h.tap(A::Mode(CalcMode::Programmer));
    h.tap(A::Digits("FF"));
    h.touch(TouchEvent::Down(Point::new(48.0, 28.0)));
    h.touch(TouchEvent::Up(Point::new(48.0, 28.0)));
    assert!(matches!(
        h.launcher.lock().unwrap().active_app,
        ActiveApp::Launcher
    ));
    h.launcher.lock().unwrap().open_app("calculator");
    assert_eq!(h.calc.lock().unwrap().programmer.value, 255);
    h.touch(TouchEvent::Down(Point::new(976.0, 28.0)));
    h.touch(TouchEvent::Up(Point::new(976.0, 28.0)));
    assert!(!h
        .launcher
        .lock()
        .unwrap()
        .running_apps
        .contains_key("calculator"));
}
#[test]
fn translated_programmer_result_and_backspace_icon_are_visible() {
    let mut h = Harness::new();
    h.tap(A::Mode(CalcMode::Programmer));
    h.tap(A::Digits("FF"));
    let layout = calculator_layout(&h.calc.lock().unwrap(), Size::new(976.0, 506.0));
    let count_ink = |rect: Rect| {
        let mut count = 0;
        for y in (rect.y + 78.0).ceil() as usize..(rect.bottom() + 78.0).floor() as usize {
            for x in (rect.x + 24.0).ceil() as usize..(rect.right() + 24.0).floor() as usize {
                if h.backend.pixels[y * 1024 + x] == tiny_flutter::theme::Folio::ink().to_rgb565() {
                    count += 1;
                }
            }
        }
        count
    };
    assert!(
        count_ink(layout.display) > 100,
        "result must survive a translated local clip"
    );
    assert!(
        count_ink(layout.key(A::Backspace).unwrap().rect) > 10,
        "vector icon belongs inside its button"
    );
}
