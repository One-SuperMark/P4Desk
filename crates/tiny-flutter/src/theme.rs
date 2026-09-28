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
