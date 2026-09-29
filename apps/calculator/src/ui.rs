use crate::{AngleUnit, BinaryOp, CalcMode, CalcState, UnaryOp, WordUnary};
use std::sync::{Arc, Mutex};
use tiny_flutter::prelude::*;

pub const CALCULATOR_BG: Color = Color::from_hex(0x292726);
const NUMBER: Color = Color::from_hex(0x787675);
const FUNCTION: Color = Color::from_hex(0x999796);
const SCIENCE: Color = Color::from_hex(0x5b5958);
const ORANGE: Color = Color::from_hex(0xff9f00);
const MUTED: Color = Color::from_hex(0xb5b1ae);

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyAction {
    Digit(char),
    Digits(&'static str),
    Dot,
    Exponent,
    Clear,
    Backspace,
    Sign,
    Percent,
    Binary(BinaryOp),
    Unary(UnaryOp),
    Open,
    Close,
    Equals,
    Second,
    Angle,
    Constant(f64),
    Random,
    MemoryClear,
    MemoryAdd,
    MemorySubtract,
    MemoryRecall,
    Word(WordUnary),
    Mode(CalcMode),
    Radix(u32),
    ToggleBinary,
    Character(bool),
    Bit(u32),
}
impl KeyAction {
    pub fn apply(self, state: &mut CalcState) {
        match self {
            Self::Digit(c) => state.input_digit(c),
            Self::Digits(s) => {
                for c in s.chars() {
                    state.input_digit(c);
                }
            }
            Self::Dot => state.input_dot(),
            Self::Exponent => state.input_exponent(),
            Self::Clear => state.clear_entry_or_all(),
            Self::Backspace => state.backspace(),
            Self::Sign => state.toggle_sign(),
            Self::Percent => state.percent(),
            Self::Binary(op) => state.binary(op),
            Self::Unary(op) => state.function(op),
            Self::Open => state.open_parenthesis(),
            Self::Close => state.close_parenthesis(),
            Self::Equals => state.calculate(),
            Self::Second => state.second_functions = !state.second_functions,
            Self::Angle => {
                state.angle = if state.angle == AngleUnit::Degrees {
                    AngleUnit::Radians
                } else {
                    AngleUnit::Degrees
                }
            }
            Self::Constant(v) => state.constant(v),
            Self::Random => state.random(),
            Self::MemoryClear => state.memory = 0.0,
            Self::MemoryAdd => state.memory_add(false),
            Self::MemorySubtract => state.memory_add(true),
            Self::MemoryRecall => state.memory_recall(),
            Self::Word(op) => state.programmer.unary(op),
            Self::Mode(mode) => state.set_mode(mode),
            Self::Radix(radix) => state.programmer.set_radix(radix),
            Self::ToggleBinary => state.programmer.show_binary = !state.programmer.show_binary,
            Self::Character(unicode) => state.programmer.unicode = unicode,
            Self::Bit(bit) => state.programmer.toggle_bit(bit),
        }
    }
}

#[derive(Clone)]
pub struct CalculatorKey {
    pub label: String,
    pub action: KeyAction,
    pub rect: Rect,
    pub enabled: bool,
    pub selected: bool,
    color: Color,
    exponent: Option<&'static str>,
    font_size: f32,
}
pub struct CalculatorLayout {
    pub keys: Vec<CalculatorKey>,
    pub keyboard: Rect,
    pub display: Rect,
    pub binary: Option<Rect>,
}
impl CalculatorLayout {
    pub fn key(&self, action: KeyAction) -> Option<&CalculatorKey> {
        self.keys.iter().find(|k| k.action == action)
    }
}
struct KeySpec {
    label: &'static str,
    action: KeyAction,
    color: Color,
    exponent: Option<&'static str>,
    selected: bool,
}
fn key(label: &'static str, action: KeyAction, color: Color) -> KeySpec {
    KeySpec {
        label,
        action,
        color,
        exponent: None,
        selected: false,
    }
}
fn power(label: &'static str, exponent: &'static str, action: KeyAction) -> KeySpec {
    KeySpec {
        exponent: Some(exponent),
        ..key(label, action, SCIENCE)
    }
}
fn basic_keys(clear: &'static str) -> Vec<KeySpec> {
    use KeyAction::*;
    vec![
        key("", Backspace, FUNCTION),
        key(clear, Clear, FUNCTION),
        key("%", Percent, FUNCTION),
        key("÷", Binary(BinaryOp::Divide), ORANGE),
        key("7", Digit('7'), NUMBER),
        key("8", Digit('8'), NUMBER),
        key("9", Digit('9'), NUMBER),
        key("×", Binary(BinaryOp::Multiply), ORANGE),
        key("4", Digit('4'), NUMBER),
        key("5", Digit('5'), NUMBER),
        key("6", Digit('6'), NUMBER),
        key("−", Binary(BinaryOp::Subtract), ORANGE),
        key("1", Digit('1'), NUMBER),
        key("2", Digit('2'), NUMBER),
        key("3", Digit('3'), NUMBER),
        key("+", Binary(BinaryOp::Add), ORANGE),
        key("+/−", Sign, NUMBER),
        key("0", Digit('0'), NUMBER),
        key(".", Dot, NUMBER),
        key("=", Equals, ORANGE),
    ]
}
fn scientific_keys(state: &CalcState) -> Vec<KeySpec> {
    use KeyAction::*;
    use UnaryOp::*;
    let second = state.second_functions;
    let mut keys = vec![
        key("(", Open, SCIENCE),
        key(")", Close, SCIENCE),
        key("mc", MemoryClear, SCIENCE),
        key("m+", MemoryAdd, SCIENCE),
        key("m−", MemorySubtract, SCIENCE),
        key("mr", MemoryRecall, SCIENCE),
        key("", Backspace, FUNCTION),
        key(state.clear_label(), Clear, FUNCTION),
        key("%", Percent, FUNCTION),
        key("÷", Binary(BinaryOp::Divide), ORANGE),
        power("2", "nd", Second),
        power("x", "2", Unary(Square)),
        power("x", "3", Unary(Cube)),
        power("x", "y", Binary(BinaryOp::Power)),
        power(
            if second { "2" } else { "e" },
            "x",
            Unary(if second { Exp2 } else { Exp }),
        ),
        power("10", "x", Unary(Exp10)),
        key("7", Digit('7'), NUMBER),
        key("8", Digit('8'), NUMBER),
        key("9", Digit('9'), NUMBER),
        key("×", Binary(BinaryOp::Multiply), ORANGE),
        key("1/x", Unary(Reciprocal), SCIENCE),
        key("√x", Unary(Sqrt), SCIENCE),
        power("√x", "3", Unary(Cbrt)),
        power("√x", "y", Binary(BinaryOp::Root)),
        key("ln", Unary(Ln), SCIENCE),
        power(
            "log",
            if second { "2" } else { "10" },
            Unary(if second { Log2 } else { Log10 }),
        ),
        key("4", Digit('4'), NUMBER),
        key("5", Digit('5'), NUMBER),
        key("6", Digit('6'), NUMBER),
        key("−", Binary(BinaryOp::Subtract), ORANGE),
        key("x!", Unary(Factorial), SCIENCE),
        key(
            if second { "asin" } else { "sin" },
            Unary(if second { Asin } else { Sin }),
            SCIENCE,
        ),
        key(
            if second { "acos" } else { "cos" },
            Unary(if second { Acos } else { Cos }),
            SCIENCE,
        ),
        key(
            if second { "atan" } else { "tan" },
            Unary(if second { Atan } else { Tan }),
            SCIENCE,
        ),
        key("e", Constant(std::f64::consts::E), SCIENCE),
        key("EE", Exponent, SCIENCE),
        key("1", Digit('1'), NUMBER),
        key("2", Digit('2'), NUMBER),
        key("3", Digit('3'), NUMBER),
        key("+", Binary(BinaryOp::Add), ORANGE),
        key("Rand", Random, SCIENCE),
        key(
            if second { "asinh" } else { "sinh" },
            Unary(if second { Asinh } else { Sinh }),
            SCIENCE,
        ),
        key(
            if second { "acosh" } else { "cosh" },
            Unary(if second { Acosh } else { Cosh }),
            SCIENCE,
        ),
        key(
            if second { "atanh" } else { "tanh" },
            Unary(if second { Atanh } else { Tanh }),
            SCIENCE,
        ),
        key("π", Constant(std::f64::consts::PI), SCIENCE),
        key(
            if state.angle == AngleUnit::Degrees {
                "Rad"
            } else {
                "Deg"
            },
            Angle,
            SCIENCE,
        ),
        key("+/−", Sign, NUMBER),
        key("0", Digit('0'), NUMBER),
        key(".", Dot, NUMBER),
        key("=", Equals, ORANGE),
    ];
    keys[10].selected = second;
    keys[5].selected = state.memory != 0.0;
    keys
}
fn programmer_keys(clear: &'static str) -> Vec<KeySpec> {
    use KeyAction::*;
    vec![
        key("", Backspace, FUNCTION),
        key("(", Open, SCIENCE),
        key(")", Close, SCIENCE),
        key("D", Digit('D'), NUMBER),
        key("E", Digit('E'), NUMBER),
        key("F", Digit('F'), NUMBER),
        key(clear, Clear, FUNCTION),
        key("AND", Binary(BinaryOp::And), SCIENCE),
        key("OR", Binary(BinaryOp::Or), SCIENCE),
        key("XOR", Binary(BinaryOp::Xor), SCIENCE),
        key("A", Digit('A'), NUMBER),
        key("B", Digit('B'), NUMBER),
        key("C", Digit('C'), NUMBER),
        key("÷", Binary(BinaryOp::Divide), ORANGE),
        key("NOR", Binary(BinaryOp::Nor), SCIENCE),
        key("<<", Word(WordUnary::ShiftLeft), SCIENCE),
        key(">>", Word(WordUnary::ShiftRight), SCIENCE),
        key("7", Digit('7'), NUMBER),
        key("8", Digit('8'), NUMBER),
        key("9", Digit('9'), NUMBER),
        key("×", Binary(BinaryOp::Multiply), ORANGE),
        key("NOT", Word(WordUnary::Not), SCIENCE),
        key("x<<y", Binary(BinaryOp::ShiftLeft), SCIENCE),
        key("x>>y", Binary(BinaryOp::ShiftRight), SCIENCE),
        key("4", Digit('4'), NUMBER),
        key("5", Digit('5'), NUMBER),
        key("6", Digit('6'), NUMBER),
        key("−", Binary(BinaryOp::Subtract), ORANGE),
        key("NEG", Word(WordUnary::Negate), SCIENCE),
        key("RoL", Word(WordUnary::RotateLeft), SCIENCE),
        key("RoR", Word(WordUnary::RotateRight), SCIENCE),
        key("1", Digit('1'), NUMBER),
        key("2", Digit('2'), NUMBER),
        key("3", Digit('3'), NUMBER),
        key("+", Binary(BinaryOp::Add), ORANGE),
        key("mod", Binary(BinaryOp::Mod), SCIENCE),
        power("flip", "8", Word(WordUnary::Flip8)),
        power("flip", "16", Word(WordUnary::Flip16)),
        key("FF", Digits("FF"), NUMBER),
        key("0", Digit('0'), NUMBER),
        key("00", Digits("00"), NUMBER),
        key("=", Equals, ORANGE),
    ]
}

/// One geometry source for drawing and touch targets, in calculator-local coordinates.
pub fn calculator_layout(state: &CalcState, size: Size) -> CalculatorLayout {
    let w = size.width.max(1.0);
    let h = size.height.max(1.0);
    let scale = (w / 976.0).min(h / 506.0).min(1.0);
    let gap = (8.0 * scale).max(4.0);
    let mode_h = 34.0 * scale.max(0.7);
    let mut keys = Vec::new();
    let (columns, rows, specs) = match state.mode {
        CalcMode::Basic => (4, 5, basic_keys(state.clear_label())),
        CalcMode::Scientific => (10, 5, scientific_keys(state)),
        CalcMode::Programmer => (7, 6, programmer_keys(state.clear_label())),
    };
    let kw = if state.mode == CalcMode::Basic {
        (h * 1.13).min(w * 0.85)
    } else {
        w
    };
    let kx = (w - kw) * 0.5;
    let ky = match state.mode {
        CalcMode::Basic => h * 0.315,
        CalcMode::Scientific => h * 0.30,
        CalcMode::Programmer => {
            if state.programmer.show_binary {
                h * 0.413
            } else {
                h * 0.30
            }
        }
    }
    .max(mode_h + 64.0 * scale);
    let bh = (h - ky - gap * (rows - 1) as f32) / rows as f32;
    let bw = (kw - gap * (columns - 1) as f32) / columns as f32;
    for (i, spec) in specs.into_iter().enumerate() {
        let enabled = if state.mode == CalcMode::Programmer {
            match spec.action {
                KeyAction::Digit(c) => state.programmer.digit_enabled(c),
                KeyAction::Digits(s) => s.chars().all(|c| state.programmer.digit_enabled(c)),
                _ => true,
            }
        } else {
            true
        };
        let fs: f32 = if columns == 4 {
            34.0
        } else if matches!(
            spec.action,
            KeyAction::Digit(_)
                | KeyAction::Equals
                | KeyAction::Binary(
                    BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Multiply | BinaryOp::Divide
                )
        ) {
            28.0
        } else {
            22.0
        };
        keys.push(CalculatorKey {
            label: spec.label.into(),
            action: spec.action,
            rect: Rect::from_ltwh(
                kx + (i % columns) as f32 * (bw + gap),
                ky + (i / columns) as f32 * (bh + gap),
                bw,
                bh,
            ),
            enabled,
            selected: spec.selected,
            color: spec.color,
            exponent: spec.exponent,
            font_size: fs.min(bh * 0.65),
        });
    }
    let mut binary = None;
    let display_bottom = if state.mode == CalcMode::Programmer {
        let controls_y = mode_h + 64.0 * scale;
        let ch = 28.0 * scale.max(0.7);
        for (label, action, x, cw, selected) in [
            (
                "ASCII",
                KeyAction::Character(false),
                0.0,
                82.0,
                !state.programmer.unicode,
            ),
            (
                "Unicode",
                KeyAction::Character(true),
                88.0,
                100.0,
                state.programmer.unicode,
            ),
            (
                if state.programmer.show_binary {
                    "隐藏二进制"
                } else {
                    "显示二进制"
                },
                KeyAction::ToggleBinary,
                208.0,
                138.0,
                false,
            ),
            (
                "8",
                KeyAction::Radix(8),
                706.0,
                72.0,
                state.programmer.radix == 8,
            ),
            (
                "10",
                KeyAction::Radix(10),
                786.0,
                84.0,
                state.programmer.radix == 10,
            ),
            (
                "16",
                KeyAction::Radix(16),
                878.0,
                98.0,
                state.programmer.radix == 16,
            ),
        ] {
            keys.push(CalculatorKey {
                label: label.into(),
                action,
                rect: Rect::from_ltwh(x * w / 976.0, controls_y, cw * w / 976.0, ch),
                enabled: true,
                selected,
                color: SCIENCE,
                exponent: None,
                font_size: 16.0 * scale.max(0.85),
            });
        }
        if state.programmer.show_binary {
            let by = controls_y + ch + 18.0 * scale;
            let row_gap = 12.0 * scale;
            let cell_h = ((ky - by - row_gap - 8.0 * scale) * 0.5).max(16.0);
            let group_gap = 9.0 * scale;
            let cell_w = (w - group_gap * 7.0) / 32.0;
            binary = Some(Rect::from_ltwh(0.0, by, w, cell_h * 2.0 + row_gap));
            for i in 0..64 {
                let bit = 63 - i as u32;
                let col = i % 32;
                let on = state.programmer.value & (1u64 << bit) != 0;
                keys.push(CalculatorKey {
                    label: if on { "1" } else { "0" }.into(),
                    action: KeyAction::Bit(bit),
                    rect: Rect::from_ltwh(
                        col as f32 * cell_w + (col / 4) as f32 * group_gap,
                        by + (i / 32) as f32 * (cell_h + row_gap),
                        cell_w,
                        cell_h,
                    ),
                    enabled: true,
                    selected: on,
                    color: CALCULATOR_BG,
                    exponent: None,
                    font_size: 16.0 * scale.max(0.85),
                });
            }
        }
        controls_y - 2.0
    } else {
        ky - 10.0 * scale
    };
    CalculatorLayout {
        keys,
        keyboard: Rect::from_ltwh(kx, ky, kw, h - ky),
        display: Rect::from_ltwh(kx, mode_h + 4.0, kw, display_bottom - mode_h - 4.0),
        binary,
    }
}

/// The launcher places this independent selector beside its close control.
pub fn mode_selector_layout(state: &CalcState, size: Size) -> Vec<CalculatorKey> {
    let gap = 8.0;
    let width = (size.width - gap * 2.0) / 3.0;
    [CalcMode::Basic, CalcMode::Scientific, CalcMode::Programmer]
        .into_iter()
        .enumerate()
        .map(|(i, mode)| CalculatorKey {
            label: mode.label().into(),
            action: KeyAction::Mode(mode),
            rect: Rect::from_ltwh(i as f32 * (width + gap), 0.0, width, size.height),
            enabled: true,
            selected: state.mode == mode,
            color: SCIENCE,
            exponent: None,
            font_size: 18.0,
        })
        .collect()
}

pub fn build_mode_selector(state: Arc<Mutex<CalcState>>, size: Size) -> impl Widget {
    let keys = mode_selector_layout(&state.lock().unwrap(), size);
    let mut row = Stack::new();
    for key in keys {
        let (x, y) = (key.rect.x, key.rect.y);
        row = row.push(
            Positioned::new(key_button(state.clone(), key))
                .left(x)
                .top(y),
        );
    }
    Container::new()
        .width(size.width)
        .height(size.height)
        .child(row)
}

struct LabelPainter {
    label: String,
    exponent: Option<&'static str>,
    size: f32,
    color: Color,
    backspace: bool,
}
impl CustomPainter for LabelPainter {
    fn paint(&self, canvas: &mut Canvas, size: Size) {
        if self.backspace {
            let side = self.size.min(size.height * 0.68);
            let x = (size.width - side) * 0.5;
            let y = (size.height - side) * 0.5;
            tiny_flutter::graphics::svg_icons_generated::UI_BACKSPACE.paint(
                canvas,
                Rect::from_ltwh(x, y, side, side),
                self.color,
            );
            return;
        }
        let font = Font::default_font();
        let fs = self.size.min(
            (size.width - 10.0) / font.measure_text(&self.label, self.size).width.max(1.0)
                * self.size,
        );
        let width = font.measure_text(&self.label, fs).width;
        let exp_size = (fs * 0.58).max(10.0);
        let exp_w = self
            .exponent
            .map_or(0.0, |s| font.measure_text(s, exp_size).width);
        // Center the visible capital bounds rather than a whole line box.
        let h = font.glyph('H', fs).height as f32;
        let origin_y = (size.height - h) * 0.5 - (font.cap_height(fs) - h);
        let x = (size.width - width - exp_w) * 0.5;
        let prefix = self.label == "√x" && self.exponent.is_some();
        canvas.draw_text(
            &self.label,
            font,
            fs,
            Point::new(x + if prefix { exp_w } else { 0.0 }, origin_y),
            self.color,
        );
        if let Some(exp) = self.exponent {
            canvas.draw_text(
                exp,
                font,
                exp_size,
                Point::new(
                    if prefix { x } else { x + width },
                    origin_y
                        + if matches!(self.label.as_str(), "log" | "flip") {
                            fs * 0.50
                        } else {
                            -fs * 0.18
                        },
                ),
                self.color,
            );
        }
    }
}
struct DisplayPainter {
    state: CalcState,
    display: Rect,
    binary: Option<Rect>,
}
impl CustomPainter for DisplayPainter {
    fn paint(&self, canvas: &mut Canvas, size: Size) {
        let font = Font::default_font();
        let primary = self.state.primary_display();
        let fs = (self.display.height * 0.62).clamp(28.0, 60.0);
        let fs = fs
            .min((self.display.width - 20.0) / font.measure_text(&primary, fs).width.max(1.0) * fs);
        let mut expression = self
            .state
            .active_error()
            .map_or_else(|| self.state.expression_display(), |e| e.message().into());
        if expression == self.state.current_input || expression == primary {
            expression.clear();
        }
        while font.measure_text(&expression, 16.0).width > self.display.width - 20.0 {
            expression.remove(0);
        }
        canvas.save();
        canvas.clip_rect(self.display);
        canvas.draw_text(
            &expression,
            font,
            16.0,
            Point::new(
                self.display.right() - 10.0 - font.measure_text(&expression, 16.0).width,
                self.display.y,
            ),
            MUTED,
        );
        let right = self.display.right()
            - if self.state.mode == CalcMode::Programmer {
                26.0
            } else {
                10.0
            };
        let bottom = self.display.bottom() - 8.0;
        canvas.draw_text(
            &primary,
            font,
            fs,
            Point::new(
                right - font.measure_text(&primary, fs).width,
                bottom - font.cap_height(fs),
            ),
            Color::WHITE,
        );
        if self.state.mode == CalcMode::Programmer {
            canvas.draw_text(
                &self.state.programmer.radix.to_string(),
                font,
                14.0,
                Point::new(right + 1.0, bottom - 5.0),
                MUTED,
            );
        } else if self.state.mode == CalcMode::Scientific && self.state.memory != 0.0 {
            canvas.draw_text(
                "M",
                font,
                16.0,
                Point::new(self.display.x + 6.0, bottom - 18.0),
                MUTED,
            );
        }
        canvas.restore();
        if let Some(binary) = self.binary {
            for (label, x, y) in [
                ("63", 0.0, binary.y - 14.0),
                ("32", size.width - 16.0, binary.y - 14.0),
                ("31", 0.0, binary.y + binary.height * 0.5 - 6.0),
                ("0", size.width - 8.0, binary.y + binary.height * 0.5 - 6.0),
            ] {
                canvas.draw_text(label, font, 10.0, Point::new(x, y), MUTED);
            }
        }
        if self.state.mode == CalcMode::Programmer {
            let value = self.state.programmer.character_display();
            canvas.draw_text(
                &value,
                font,
                16.0,
                Point::new(8.0, self.display.y + 8.0),
                MUTED,
            );
        }
    }
}

pub fn build_calculator_ui(state: Arc<Mutex<CalcState>>, size: Size) -> impl Widget {
    let snapshot = state.lock().unwrap().clone();
    let layout = calculator_layout(&snapshot, size);
    let mut root = Stack::new()
        .push(
            Container::new()
                .width(size.width)
                .height(size.height)
                .color(CALCULATOR_BG),
        )
        .push(
            CustomPaint::new(DisplayPainter {
                state: snapshot,
                display: layout.display,
                binary: layout.binary,
            })
            .size(size),
        );
    for key in layout.keys {
        let (x, y) = (key.rect.x, key.rect.y);
        root = root.push(
            Positioned::new(key_button(state.clone(), key))
                .left(x)
                .top(y),
        );
    }
    Container::new()
        .width(size.width)
        .height(size.height)
        .child(root)
}

fn key_button(state: Arc<Mutex<CalcState>>, key: CalculatorKey) -> ElevatedButton {
    let bit = matches!(key.action, KeyAction::Bit(_));
    let orange_selection = matches!(key.action, KeyAction::Radix(_) | KeyAction::Mode(_));
    let color = if bit {
        CALCULATOR_BG
    } else if !key.enabled {
        Color::from_hex(0x403e3d)
    } else if key.selected {
        if orange_selection {
            ORANGE
        } else {
            FUNCTION
        }
    } else {
        key.color
    };
    let label_color = if !key.enabled {
        Color::from_hex(0x74716f)
    } else if bit && key.selected {
        ORANGE
    } else {
        Color::WHITE
    };
    let label = CustomPaint::new(LabelPainter {
        label: key.label,
        exponent: key.exponent,
        size: key.font_size,
        color: label_color,
        backspace: key.action == KeyAction::Backspace,
    })
    .size(Size::new(key.rect.width, key.rect.height));
    let mut button = ElevatedButton::new(label).style(
        ButtonStyle::new()
            .size(key.rect.width, key.rect.height)
            .color(color)
            .pressed_color(if bit {
                FUNCTION
            } else {
                Color::from_hex(0xc7c2be)
            })
            .border_radius(if bit { 4.0 } else { key.rect.height * 0.5 })
            .antialias(true)
            .padding(EdgeInsets::all(0.0)),
    );
    if key.enabled {
        let state = state.clone();
        let action = key.action;
        button = button.on_pressed(move || {
            if let Ok(mut s) = state.lock() {
                action.apply(&mut s);
            }
        });
    }
    button
}
