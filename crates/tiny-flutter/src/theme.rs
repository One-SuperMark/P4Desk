use crate::graphics::color::Color;
use crate::widgets::button::ButtonStyle;

/// Color palette configuration for a tiny-flutter application or component tree.
#[derive(Clone, Debug, PartialEq)]
pub struct ColorScheme {
    /// Screen or canvas background color.
    pub background: Color,
    /// Card or container surface color.
    pub surface: Color,
    /// Border color for surfaces and cards.
    pub surface_border: Color,
    /// Primary accent color (e.g. operators, active elements).
    pub primary: Color,
    /// Secondary accent color (e.g. highlights, equals button).
    pub secondary: Color,
    /// Primary high-contrast text color.
    pub text_primary: Color,
    /// Secondary muted text color (e.g. captions, history expression).
    pub text_secondary: Color,

    // Button specific palette
    pub btn_function: Color,
    pub btn_function_pressed: Color,
    pub btn_function_text: Color,

    pub btn_number: Color,
    pub btn_number_pressed: Color,
    pub btn_number_text: Color,

    pub btn_operator: Color,
    pub btn_operator_pressed: Color,
    pub btn_operator_text: Color,

    pub btn_equals: Color,
    pub btn_equals_pressed: Color,
    pub btn_equals_text: Color,
}

/// Comprehensive theme data specifying visual styling across components.
#[derive(Clone, Debug, PartialEq)]
pub struct ThemeData {
    pub color_scheme: ColorScheme,
    pub card_border_width: f32,
}

impl Default for ThemeData {
    fn default() -> Self {
        Self::amoled()
    }
}

impl ThemeData {
    /// Pure AMOLED Ultra-Dark Theme (zero backlight power on OLED displays, based on iPhone dark calculator).
    pub fn amoled() -> Self {
        Self::ios_dark()
    }

    /// Official Apple iPhone Dark Calculator Theme Palette.
    pub fn ios_dark() -> Self {
        Self {
            color_scheme: ColorScheme {
                background: Color::from_rgb(0, 0, 0), // #000000 True Black
                surface: Color::from_rgb(28, 28, 30), // #1C1C1E Apple System Gray 6
                surface_border: Color::from_rgb(44, 44, 46), // #2C2C2E Apple Dark Separator
                primary: Color::from_rgb(255, 159, 10), // #FF9F0A Apple System Orange
                secondary: Color::from_rgb(255, 159, 10), // #FF9F0A Equals matches Operator
                text_primary: Color::WHITE,           // #FFFFFF
                text_secondary: Color::from_rgb(142, 142, 147), // #8E8E93 Apple System Gray

                // Function Keys (AC / C, +/-, %): Apple Light Gray (#A5A5A5) with black text
                btn_function: Color::from_rgb(165, 165, 165),
                btn_function_pressed: Color::from_rgb(217, 217, 217),
                btn_function_text: Color::BLACK,

                // Number Keys (0-9, .): Apple Dark Gray (#333333) with white text
                btn_number: Color::from_rgb(51, 51, 51),
                btn_number_pressed: Color::from_rgb(115, 115, 115),
                btn_number_text: Color::WHITE,

                // Operator Keys (÷, ×, -, +): Apple Orange (#FF9F0A) with white text
                btn_operator: Color::from_rgb(255, 159, 10),
                btn_operator_pressed: Color::from_rgb(204, 127, 8),
                btn_operator_text: Color::WHITE,

                // Equals Key (=): Matches operator color (#FF9F0A) with white text
                btn_equals: Color::from_rgb(255, 159, 10),
                btn_equals_pressed: Color::from_rgb(204, 127, 8),
                btn_equals_text: Color::WHITE,
            },
            card_border_width: 1.0,
        }
    }

    /// Cyber Blue Tech Dark Theme.
    pub fn cyber_blue() -> Self {
        Self {
            color_scheme: ColorScheme {
                background: Color::from_rgb(10, 14, 26),
                surface: Color::from_rgb(18, 26, 46),
                surface_border: Color::from_rgb(36, 52, 88),
                primary: Color::from_rgb(0, 180, 255), // Electric Sky
                secondary: Color::from_rgb(0, 230, 180), // Neo Mint
                text_primary: Color::from_rgb(240, 246, 255),
                text_secondary: Color::from_rgb(120, 150, 195),

                btn_function: Color::from_rgb(28, 42, 70),
                btn_function_pressed: Color::from_rgb(42, 62, 100),
                btn_function_text: Color::from_rgb(140, 200, 255),

                btn_number: Color::from_rgb(20, 30, 52),
                btn_number_pressed: Color::from_rgb(32, 48, 80),
                btn_number_text: Color::from_rgb(230, 240, 255),

                btn_operator: Color::from_rgb(0, 160, 240),
                btn_operator_pressed: Color::from_rgb(0, 130, 200),
                btn_operator_text: Color::WHITE,

                btn_equals: Color::from_rgb(0, 220, 170),
                btn_equals_pressed: Color::from_rgb(0, 180, 140),
                btn_equals_text: Color::from_rgb(10, 20, 30),
            },
            card_border_width: 1.5,
        }
    }

    /// Minimalist Clean Light Theme.
    pub fn light() -> Self {
        Self {
            color_scheme: ColorScheme {
                background: Color::from_rgb(242, 244, 248),
                surface: Color::WHITE,
                surface_border: Color::from_rgb(215, 220, 232),
                primary: Color::from_rgb(249, 115, 22), // Orange Accent
                secondary: Color::from_rgb(37, 99, 235), // Royal Blue
                text_primary: Color::from_rgb(24, 30, 42),
                text_secondary: Color::from_rgb(100, 110, 130),

                btn_function: Color::from_rgb(228, 232, 240),
                btn_function_pressed: Color::from_rgb(208, 214, 225),
                btn_function_text: Color::from_rgb(50, 65, 90),

                btn_number: Color::WHITE,
                btn_number_pressed: Color::from_rgb(235, 238, 245),
                btn_number_text: Color::from_rgb(24, 30, 42),

                btn_operator: Color::from_rgb(249, 115, 22),
                btn_operator_pressed: Color::from_rgb(220, 95, 12),
                btn_operator_text: Color::WHITE,

                btn_equals: Color::from_rgb(37, 99, 235),
                btn_equals_pressed: Color::from_rgb(28, 78, 190),
                btn_equals_text: Color::WHITE,
            },
            card_border_width: 1.0,
        }
    }

    // Direct convenience property accessors
    pub fn background(&self) -> Color {
        self.color_scheme.background
    }

    pub fn surface(&self) -> Color {
        self.color_scheme.surface
    }

    pub fn surface_border(&self) -> Color {
        self.color_scheme.surface_border
    }

    pub fn text_primary(&self) -> Color {
        self.color_scheme.text_primary
    }

    pub fn text_secondary(&self) -> Color {
        self.color_scheme.text_secondary
    }

    // Standardized ButtonStyle generators
    pub fn function_button_style(&self, width: f32, height: f32, radius: f32) -> ButtonStyle {
        ButtonStyle::new()
            .color(self.color_scheme.btn_function)
            .pressed_color(self.color_scheme.btn_function_pressed)
            .size(width, height)
            .border_radius(radius)
    }

    pub fn number_button_style(&self, width: f32, height: f32, radius: f32) -> ButtonStyle {
        ButtonStyle::new()
            .color(self.color_scheme.btn_number)
            .pressed_color(self.color_scheme.btn_number_pressed)
            .size(width, height)
            .border_radius(radius)
    }

    pub fn operator_button_style(&self, width: f32, height: f32, radius: f32) -> ButtonStyle {
        ButtonStyle::new()
            .color(self.color_scheme.btn_operator)
            .pressed_color(self.color_scheme.btn_operator_pressed)
            .size(width, height)
            .border_radius(radius)
    }

    pub fn equals_button_style(&self, width: f32, height: f32, radius: f32) -> ButtonStyle {
        ButtonStyle::new()
            .color(self.color_scheme.btn_equals)
            .pressed_color(self.color_scheme.btn_equals_pressed)
            .size(width, height)
            .border_radius(radius)
    }
}

// Folio-inspired design tokens. See third_party/folio/LICENSE and docs/folio-ui.md.
thread_local! {
    static APPEARANCE: std::cell::Cell<(bool, u8)> = const { std::cell::Cell::new((false, 65)) };
    static BACKDROP: std::cell::Cell<Option<GlassBackdrop>> = const { std::cell::Cell::new(None) };
}
#[derive(Clone, Copy)]
pub struct GlassBackdrop {
    /// Immutable, row-major RGB888 prefiltered wallpaper pixels.
    pub pixels: &'static [u8],
    pub width: u32,
    pub height: u32,
}

/// Each surface selects its own source. App pages must never sample the desktop.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GlassMaterial {
    Page,
    Desktop,
    DesktopCard,
}

pub struct Folio;
impl Folio {
    pub fn on_fill(fill: Color) -> Color {
        if (fill.r as u32 * 299 + fill.g as u32 * 587 + fill.b as u32 * 114) >= 145_000 {
            Color::from_hex(0x17212e)
        } else {
            Color::WHITE
        }
    }
    pub fn set_backdrop(backdrop: GlassBackdrop) {
        BACKDROP.with(|b| b.set(Some(backdrop)));
    }
    pub fn backdrop() -> Option<GlassBackdrop> {
        BACKDROP.with(|b| b.get())
    }
    /// UI-thread local: worker threads and parallel headless tests cannot alter a frame's palette.
    pub fn configure(light: bool, glass: u8) {
        APPEARANCE.with(|v| v.set((light, glass.min(100))));
    }
    pub fn is_light() -> bool {
        APPEARANCE.with(|v| v.get().0)
    }
    pub fn glass_amount() -> u8 {
        APPEARANCE.with(|v| v.get().1)
    }
    pub fn pick(dark: Color, light: Color) -> Color {
        if Self::is_light() {
            light
        } else {
            dark
        }
    }
    pub fn bg() -> Color {
        Self::pick(Self::BG, Color::from_hex(0xeef1f5))
    }
    pub fn surface() -> Color {
        Self::pick(Self::SURFACE, Color::from_hex(0xffffff))
    }
    pub fn raised() -> Color {
        Self::pick(Self::RAISED, Color::from_hex(0xe0e5ec))
    }
    pub fn line() -> Color {
        Self::pick(Self::LINE, Color::from_hex(0xc8cfd8))
    }
    pub fn ink() -> Color {
        Self::pick(Self::INK, Color::from_hex(0x17212e))
    }
    pub fn muted() -> Color {
        Self::pick(Self::MUTED, Color::from_hex(0x586474))
    }
    pub fn accent() -> Color {
        Self::pick(Self::ACCENT, Color::from_hex(0xbddde0))
    }
    pub fn accent_ink() -> Color {
        Self::pick(Self::ACCENT_INK, Color::from_hex(0x17656a))
    }
    pub fn blue() -> Color {
        Self::pick(Self::BLUE, Color::from_hex(0x0067cc))
    }
    pub fn green() -> Color {
        Self::pick(Self::GREEN, Color::from_hex(0x137c3a))
    }
    pub fn orange() -> Color {
        Self::pick(Self::ORANGE, Color::from_hex(0xf49500))
    }
    pub fn red() -> Color {
        Self::pick(Self::RED, Color::from_hex(0xc43b39))
    }
    pub fn destructive() -> Color {
        Self::pick(Self::DESTRUCTIVE, Color::from_hex(0xb52f28))
    }
    pub fn destructive_fill() -> Color {
        Self::pick(Self::DESTRUCTIVE_FILL, Color::from_hex(0xf8dedb))
    }
    pub fn disabled() -> Color {
        Self::pick(Self::DISABLED, Color::from_hex(0x7b8490))
    }
    pub fn glass() -> Color {
        Self::pick(Self::GLASS, Color::from_rgba(239, 247, 255, 172))
    }
    pub fn glass_edge() -> Color {
        Self::pick(Self::GLASS_EDGE, Color::from_rgba(255, 255, 255, 150))
    }
    pub fn separator() -> Color {
        Self::pick(Self::SEPARATOR, Color::from_rgba(28, 43, 60, 30))
    }

    pub const BG: Color = Color::from_hex(0x101013);
    pub const SURFACE: Color = Color::from_hex(0x1c1c1e);
    pub const RAISED: Color = Color::from_hex(0x2c2c2e);
    pub const LINE: Color = Color::from_hex(0x3a3a3c);
    pub const INK: Color = Color::from_hex(0xf5f5f7);
    pub const MUTED: Color = Color::from_hex(0xa6a6ae);
    pub const ACCENT: Color = Color::from_hex(0x2a6a70);
    pub const ACCENT_INK: Color = Color::from_hex(0x6db7b4);
    pub const BLUE: Color = Color::from_hex(0x0a6fd6);
    pub const GREEN: Color = Color::from_hex(0x30d158);
    pub const ORANGE: Color = Color::from_hex(0xff9f0a);
    pub const RED: Color = Color::from_hex(0xff6961);
    pub const GLASS: Color = Color::from_rgba(36, 42, 49, 174);
    pub const GLASS_EDGE: Color = Color::from_rgba(255, 255, 255, 26);
    pub const CONTROL_RADIUS: f32 = 14.0;
    pub const GROUP_RADIUS: f32 = 20.0;
    pub const CHIP_RADIUS: f32 = 10.0;
    pub const CARD_RADIUS: f32 = 24.0;
    pub const SEPARATOR: Color = Color::from_rgba(255, 255, 255, 22);
    pub const DESTRUCTIVE: Color = Color::from_hex(0xff8a80);
    pub const DESTRUCTIVE_FILL: Color = Color::from_hex(0x723a38);
    pub const DISABLED: Color = Color::from_hex(0x71717a);

    /// Retain the control's hue and geometry while providing press feedback.
    pub fn pressed(fill: Color) -> Color {
        if fill.a == 0 {
            return Color::from_rgba(255, 255, 255, 24);
        }
        let lift = |channel: u8| {
            if Self::is_light() {
                (channel as u16 * 92 / 100) as u8
            } else {
                channel + ((255 - channel) as u16 * 12 / 100) as u8
            }
        };
        Color::from_rgba(lift(fill.r), lift(fill.g), lift(fill.b), fill.a)
    }
    pub fn control_style(width: f32, height: f32, fill: Color) -> ButtonStyle {
        ButtonStyle::new()
            .antialias(true)
            .size(width, height)
            .color(fill)
            .pressed_color(Self::pressed(fill))
            .border_radius(Self::CONTROL_RADIUS.min(height * 0.5))
    }
    pub fn navigation_style() -> ButtonStyle {
        Self::control_style(48.0, 48.0, Self::raised())
            .glass(true)
            .border_radius(24.0)
            .padding(crate::graphics::geometry::EdgeInsets::all(12.0))
    }
}
