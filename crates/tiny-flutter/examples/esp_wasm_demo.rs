use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tiny_flutter::prelude::*;

fn build_ui(counter: Arc<AtomicU32>) -> impl Widget {
    let count = counter.load(Ordering::Relaxed);
    let counter_clone = counter.clone();

    Container::new()
        .color(Color::from_rgb(15, 15, 20))
        .padding(EdgeInsets::all(20.0))
        .child(
            Column::new()
                .main_axis_alignment(MainAxisAlignment::Center)
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .push(
                    // Title Bar
                    Container::new()
                        .color(Color::from_rgb(30, 30, 45))
                        .padding(EdgeInsets::symmetric(10.0, 20.0))
                        .border_radius(12.0)
                        .child(
                            Text::new("ESP32 Rust · TinyFlutter")
                                .font_size(18.0)
                                .color(Color::WHITE),
                        ),
                )
                .push(SizedBox::square(24.0))
                .push(
                    // Counter Display Box
                    Container::new()
                        .color(Color::from_rgb(25, 25, 35))
                        .padding(EdgeInsets::all(24.0))
                        .border_radius(16.0)
                        .border(Color::from_rgb(45, 45, 65), 2.0)
                        .child(
                            Column::new()
                                .main_axis_alignment(MainAxisAlignment::Center)
                                .cross_axis_alignment(CrossAxisAlignment::Center)
                                .push(
                                    Text::new("Touch Counter")
                                        .font_size(14.0)
                                        .color(Color::from_rgb(150, 150, 170)),
                                )
                                .push(SizedBox::square(8.0))
                                .push(
                                    Text::new(format!("{:02}", count))
                                        .font_size(44.0)
                                        .color(Color::CYAN),
                                )
                                .push(SizedBox::square(16.0))
                                .push(
                                    ElevatedButton::new(
                                        Text::new("Tap to Increment")
                                            .font_size(14.0)
                                            .color(Color::WHITE),
                                    )
                                    .color(Color::PRIMARY)
                                    .pressed_color(Color::from_rgb(21, 101, 192))
                                    .border_radius(8.0)
                                    .padding(EdgeInsets::symmetric(10.0, 18.0))
                                    .on_pressed(move || {
                                        let next = counter_clone.fetch_add(1, Ordering::SeqCst) + 1;
                                        println!("[WASM] Button pressed! Counter: {}", next);
                                    }),
                                ),
                        ),
                )
                .push(SizedBox::square(16.0))
                .push(
                    Text::new("480x480 AMOLED · tiny-skia")
                        .font_size(12.0)
                        .color(Color::from_rgb(90, 90, 110)),
                ),
        )
}

fn main() {
    println!("[WASM] TinyFlutter Application Starting on ESP-WASMachine...");

    let screen_w = 480;
    let screen_h = 480;

    let mut backend = EspWasmBackend::new(screen_w, screen_h);
    let counter = Arc::new(AtomicU32::new(0));

    let mut app = App::new(
        build_ui(counter.clone()),
        Size::new(screen_w as f32, screen_h as f32),
    );

    let mut last_count = counter.load(Ordering::Relaxed);

    loop {
        // Check if state changed from callbacks
        let cur_count = counter.load(Ordering::Relaxed);
        if cur_count != last_count {
            app.set_root(build_ui(counter.clone()));
            last_count = cur_count;
        }

        // Run framework step (process touch, re-render dirty rects, flush)
        app.step(&mut backend);

        // Frame pacing (~50 FPS)
        std::thread::sleep(Duration::from_millis(20));
    }
}
