use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use tiny_flutter::prelude::*;

fn build_ui(counter: Arc<AtomicU32>) -> impl Widget {
    let count = counter.load(Ordering::Relaxed);
    let counter_clone = counter.clone();

    Container::new()
        .color(Color::from_rgb(18, 18, 24))
        .padding(EdgeInsets::all(20.0))
        .child(
            Column::new()
                .main_axis_alignment(MainAxisAlignment::Center)
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .push(
                    // App Bar
                    Container::new()
                        .color(Color::from_rgb(33, 33, 44))
                        .padding(EdgeInsets::symmetric(12.0, 24.0))
                        .border_radius(12.0)
                        .child(
                            Text::new("TinyFlutter Embedded")
                                .font_size(20.0)
                                .color(Color::WHITE),
                        ),
                )
                .push(SizedBox::square(24.0))
                .push(
                    // Counter Card
                    Container::new()
                        .color(Color::from_rgb(26, 26, 36))
                        .padding(EdgeInsets::all(24.0))
                        .border_radius(16.0)
                        .border(Color::from_rgb(45, 45, 60), 2.0)
                        .child(
                            Column::new()
                                .main_axis_alignment(MainAxisAlignment::Center)
                                .cross_axis_alignment(CrossAxisAlignment::Center)
                                .push(
                                    Text::new("You have pushed the button:")
                                        .font_size(14.0)
                                        .color(Color::from_rgb(160, 160, 180)),
                                )
                                .push(SizedBox::square(12.0))
                                .push(
                                    Text::new(format!("{}", count))
                                        .font_size(48.0)
                                        .color(Color::CYAN),
                                )
                                .push(SizedBox::square(20.0))
                                .push(
                                    ElevatedButton::new(
                                        Text::new("+ Increment Count")
                                            .font_size(15.0)
                                            .color(Color::WHITE),
                                    )
                                    .color(Color::PRIMARY)
                                    .pressed_color(Color::from_rgb(21, 101, 192))
                                    .border_radius(10.0)
                                    .padding(EdgeInsets::symmetric(12.0, 20.0))
                                    .on_pressed(move || {
                                        counter_clone.fetch_add(1, Ordering::SeqCst);
                                        println!(
                                            "[Flutter] Counter tapped! Current count: {}",
                                            counter_clone.load(Ordering::SeqCst)
                                        );
                                    }),
                                ),
                        ),
                )
                .push(SizedBox::square(20.0))
                .push(
                    Text::new("Pure tiny-skia 2D Software Rasterizer")
                        .font_size(12.0)
                        .color(Color::from_rgb(100, 100, 120)),
                ),
        )
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting tiny-flutter desktop simulator preview (480x480)...");

    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();

    run_simulator(
        "TinyFlutter 480x480 Embedded Preview",
        480,
        480,
        move || build_ui(counter_clone.clone()),
    )?;

    Ok(())
}
