use app_launcher::app_icons::get_app_icon_asset;
use tiny_flutter::graphics::svg_icons_generated::ALL_VECTOR_ICONS;
use tiny_flutter::prelude::*;

fn draw(icon: &VectorIcon, side: usize, clip: Option<Rect>) -> Vec<u16> {
    let mut pixmap = tiny_gfx::Pixmap565::new(side as u32, side as u32).unwrap();
    let mut canvas = Canvas::new(pixmap.as_mut());
    if let Some(clip) = clip {
        canvas.clip_rect(clip);
    }
    icon.paint(
        &mut canvas,
        Rect::from_ltwh(0.0, 0.0, side as f32, side as f32),
        Color::WHITE,
    );
    pixmap.data().to_vec()
}
#[test]
fn all_svg_icons_render_at_small_navigation_and_large_desktop_sizes() {
    assert_eq!(ALL_VECTOR_ICONS.len(), 41);
    for (name, icon) in ALL_VECTOR_ICONS {
        for side in [24, 32, 146] {
            let pixels = draw(icon, side, None);
            assert!(pixels.iter().any(|p| *p != 0), "{name} empty at {side}px");
            assert!(
                pixels.iter().any(|p| *p == 0),
                "{name} should keep outer transparent margin"
            );
        }
    }
}
#[test]
fn eight_desktop_svgs_keep_round_transparent_corners_and_smooth_edges_at_every_size() {
    for id in [
        "clock",
        "timer",
        "notes",
        "calculator",
        "mac",
        "settings",
        "display",
        "screen",
    ] {
        let icon = get_app_icon_asset(id).unwrap();
        for side in [32, 96, 146, 193] {
            let pixels = draw(icon, side, None);
            for i in [0, side - 1, side * (side - 1), side * side - 1] {
                assert_eq!(pixels[i], 0);
            }
            // A scaled vector outline gets fresh coverage rather than repeated source pixels.
            let mid = side / 2;
            let edge = &pixels[..mid * side];
            assert!(edge.iter().any(|p| *p != 0 && *p != 0xffff));
        }
    }
    assert!(get_app_icon_asset("unknown").is_none());
}
#[test]
fn vector_desktop_partial_redraw_equals_the_same_region_of_a_full_render() {
    let clip = Rect::from_ltwh(35.0, 29.0, 60.0, 72.0);
    for id in ["settings", "clock", "display"] {
        let icon = get_app_icon_asset(id).unwrap();
        let full = draw(icon, 146, None);
        let partial = draw(icon, 146, Some(clip));
        for y in 0..146 {
            for x in 0..146 {
                assert_eq!(
                    partial[y * 146 + x],
                    if (35..95).contains(&x) && (29..101).contains(&y) {
                        full[y * 146 + x]
                    } else {
                        0
                    }
                );
            }
        }
    }
}
#[test]
fn upstream_icon_widget_uses_requested_size_and_preserves_dynamic_tint() {
    let icon = Icon::new(ICON_PLAY_CIRCLE)
        .size(Size::new(96.0, 96.0))
        .color(Color::from_hex(0xff9f00));
    let mut backend = app_launcher::headless::HeadlessBackend::new(96, 96);
    App::new(icon, Size::new(96.0, 96.0)).step(&mut backend);
    assert_eq!(
        backend.pixels[48 * 96 + 48],
        Color::from_hex(0xff9f00).to_rgb565()
    );
    assert!(
        backend.pixels[48 * 96 + 8] != 0,
        "outline must scale to 96px rather than stay 48px wide"
    );
}
