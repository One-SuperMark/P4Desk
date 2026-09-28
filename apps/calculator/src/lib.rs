use std::sync::{Arc, Mutex};
use tiny_flutter::prelude::*;

#[derive(Debug, Clone)]
pub struct CalcState {
    pub current_input: String,
    pub first_operand: Option<f64>,
    pub first_operand_str: String,
    pub operator: Option<char>,
    pub secondary: String,
    pub just_calculated: bool,
}

impl Default for CalcState {
    fn default() -> Self {
        Self::new()
    }
}

impl CalcState {
    pub fn new() -> Self {
        Self {
            current_input: "0".to_string(),
            first_operand: None,
            first_operand_str: String::new(),
            operator: None,
            secondary: String::new(),
            just_calculated: false,
        }
    }

    pub fn theme(&self) -> ThemeData {
        ThemeData::amoled()
    }

    pub fn input_digit(&mut self, digit: char) {
        if self.just_calculated {
            self.current_input = digit.to_string();
            self.just_calculated = false;
            self.secondary.clear();
        } else if self.current_input == "0" {
            self.current_input = digit.to_string();
        } else if self.current_input.len() < 12 {
            self.current_input.push(digit);
        }
    }

    pub fn input_dot(&mut self) {
        if self.just_calculated {
            self.current_input = "0.".to_string();
            self.just_calculated = false;
            self.secondary.clear();
        } else if self.current_input.is_empty() {
            self.current_input = "0.".to_string();
        } else if !self.current_input.contains('.') {
            self.current_input.push('.');
        }
    }

    pub fn input_operator(&mut self, op: char) {
        let current_val: f64 = self.current_input.parse().unwrap_or(0.0);

        if let (Some(first), Some(prev_op)) = (self.first_operand, self.operator) {
            if !self.current_input.is_empty() && !self.just_calculated {
                let res = eval_op(first, prev_op, current_val);
                self.secondary = format!(
                    "{}\u{200A}{}\u{200A}{}",
                    add_commas(&self.first_operand_str),
                    prev_op,
                    add_commas(&self.current_input)
                );
                self.first_operand = Some(res);
                self.first_operand_str = format_raw_number(res);
            }
        } else {
            self.first_operand = Some(current_val);
            self.first_operand_str = if self.current_input.is_empty() {
                "0".to_string()
            } else {
                self.current_input.clone()
            };
        }

        self.operator = Some(op);
        self.current_input.clear();
        self.just_calculated = false;
    }

    pub fn calculate(&mut self) {
        if let (Some(first), Some(op)) = (self.first_operand, self.operator) {
            let second_val: f64 = if self.current_input.is_empty() {
                first
            } else {
                self.current_input.parse().unwrap_or(0.0)
            };
            let second_str = if self.current_input.is_empty() {
                self.first_operand_str.clone()
            } else {
                self.current_input.clone()
            };

            let res = eval_op(first, op, second_val);
            self.secondary = format!(
                "{}\u{200A}{}\u{200A}{}",
                add_commas(&self.first_operand_str),
                op,
                add_commas(&second_str)
            );
            self.current_input = format_raw_number(res);
            self.first_operand = None;
            self.first_operand_str.clear();
            self.operator = None;
            self.just_calculated = true;
        }
    }

    pub fn clear(&mut self) {
        self.current_input = "0".to_string();
        self.first_operand = None;
        self.first_operand_str.clear();
        self.operator = None;
        self.secondary.clear();
        self.just_calculated = false;
    }

    pub fn toggle_sign(&mut self) {
        if self.current_input != "0" && !self.current_input.is_empty() {
            if self.current_input.starts_with('-') {
                self.current_input.remove(0);
            } else {
                self.current_input.insert(0, '-');
            }
        }
    }

    pub fn percent(&mut self) {
        let val: f64 = self.current_input.parse().unwrap_or(0.0);
        let res = val / 100.0;
        self.current_input = format_raw_number(res);
    }

    pub fn primary_display(&self) -> String {
        if self.just_calculated {
            add_commas(&self.current_input)
        } else if let Some(op) = self.operator {
            if self.current_input.is_empty() {
                format!("{}\u{200A}{}", add_commas(&self.first_operand_str), op)
            } else {
                format!(
                    "{}\u{200A}{}\u{200A}{}",
                    add_commas(&self.first_operand_str),
                    op,
                    add_commas(&self.current_input)
                )
            }
        } else {
            add_commas(&self.current_input)
        }
    }

    pub fn secondary_display(&self) -> &str {
        if self.secondary.is_empty() {
            " "
        } else {
            &self.secondary
        }
    }
}

pub fn eval_op(first: f64, op: char, second: f64) -> f64 {
    match op {
        '+' => first + second,
        '-' => first - second,
        '×' => first * second,
        '÷' => {
            if second == 0.0 {
                0.0
            } else {
                first / second
            }
        }
        _ => second,
    }
}

pub fn format_raw_number(n: f64) -> String {
    if n.is_nan() || n.is_infinite() {
        return "Error".to_string();
    }
    if n.fract() == 0.0 && n.abs() < 1e12 {
        format!("{:.0}", n)
    } else {
        let raw = format!("{:.6}", n);
        raw.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

pub fn add_commas(s: &str) -> String {
    if s.is_empty() || s == "Error" {
        return s.to_string();
    }
    let is_negative = s.starts_with('-');
    let raw = if is_negative { &s[1..] } else { s };

    let parts: Vec<&str> = raw.split('.').collect();
    let int_part = parts[0];
    let frac_part = if parts.len() > 1 {
        Some(parts[1])
    } else {
        None
    };

    let mut formatted_int = String::with_capacity(int_part.len() + int_part.len() / 3);
    let len = int_part.len();
    for (i, ch) in int_part.chars().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            formatted_int.push(',');
        }
        formatted_int.push(ch);
    }

    let mut res = String::new();
    if is_negative {
        res.push('-');
    }
    res.push_str(&formatted_int);
    if let Some(frac) = frac_part {
        res.push('.');
        res.push_str(frac);
    }
    res
}

pub fn calc_btn(
    label: &'static str,
    font_size: f32,
    style: &ButtonStyle,
    text_color: Color,
    state: Arc<Mutex<CalcState>>,
    action: impl Fn(&mut CalcState) + Send + Sync + 'static,
) -> impl Widget {
    let action = Arc::new(action);
    let state_clone = state.clone();

    ElevatedButton::new(Text::new(label).font_size(font_size).color(text_color))
        .style(style.clone())
        .on_pressed(move || {
            if let Ok(mut s) = state_clone.lock() {
                action(&mut s);
            }
        })
}

pub fn build_calculator_ui(state: Arc<Mutex<CalcState>>, screen_size: Size) -> impl Widget {
    let screen_w = screen_size.width;
    let screen_h = screen_size.height;

    let (display_text, history_text, theme) = {
        let s = state.lock().unwrap();
        (
            s.primary_display(),
            s.secondary_display().to_string(),
            s.theme(),
        )
    };

    // Responsive proportional geometry:
    // Scale all dimensions dynamically relative to screen bounds:
    let min_dim = screen_w.min(screen_h);
    let keypad_w = (min_dim * 0.835).round();
    let col_gap = (keypad_w * 0.040).round().clamp(6.0, 18.0);
    let row_gap = (screen_h * 0.021).round().clamp(5.0, 14.0);

    // Exact 4-column math: 4 * btn_w + 3 * col_gap = actual_keypad_w
    let btn_w = ((keypad_w - 3.0 * col_gap) / 4.0).floor();
    let zero_w = 2.0 * btn_w + col_gap;
    let actual_keypad_w = 4.0 * btn_w + 3.0 * col_gap;

    let btn_h = (screen_h * 0.108).round().clamp(30.0, 68.0);
    let card_h = (screen_h * 0.175).round().clamp(48.0, 110.0);
    let btn_font_size = (btn_h * 0.42).round().clamp(14.0, 26.0);
    let btn_radius = (btn_h * 0.31).round().clamp(8.0, 20.0);
    let card_radius = btn_radius.clamp(10.0, 22.0);

    let card_pad_h = (actual_keypad_w * 0.045).round().clamp(10.0, 24.0);
    let card_pad_v = (card_h * 0.12).round().clamp(6.0, 16.0);
    let max_display_w = actual_keypad_w - 2.0 * card_pad_h;

    let font = Font::default_font();
    let base_display_font = (card_h * 0.41).round().clamp(20.0, 42.0);
    let sec_font_size = (base_display_font * 0.42).round().clamp(11.0, 18.0);

    let primary_font_size =
        if font.measure_text(&display_text, base_display_font).width <= max_display_w {
            base_display_font
        } else if font
            .measure_text(&display_text, base_display_font * 0.82)
            .width
            <= max_display_w
        {
            base_display_font * 0.82
        } else if font
            .measure_text(&display_text, base_display_font * 0.70)
            .width
            <= max_display_w
        {
            base_display_font * 0.70
        } else if font
            .measure_text(&display_text, base_display_font * 0.58)
            .width
            <= max_display_w
        {
            base_display_font * 0.58
        } else {
            base_display_font * 0.48
        };

    // Button styles generated directly from active ThemeData
    let fn_style = theme.function_button_style(btn_w, btn_h, btn_radius);
    let num_style = theme.number_button_style(btn_w, btn_h, btn_radius);
    let op_style = theme.operator_button_style(btn_w, btn_h, btn_radius);
    let eq_style = theme.equals_button_style(btn_w, btn_h, btn_radius);
    let zero_style = num_style.clone().width(zero_w);

    let fn_text_color = theme.color_scheme.btn_function_text;
    let num_text_color = theme.color_scheme.btn_number_text;
    let op_text_color = theme.color_scheme.btn_operator_text;
    let eq_text_color = theme.color_scheme.btn_equals_text;

    Container::new().width(screen_w).height(screen_h).child(
        col![
            // 1. Result & History Display Card
            Container::new()
                .width(actual_keypad_w)
                .height(card_h)
                .color(theme.surface())
                .padding(EdgeInsets::symmetric(card_pad_v, card_pad_h))
                .border_radius(card_radius)
                .border(theme.surface_border(), theme.card_border_width)
                .child(
                    col![
                        Text::new(&history_text)
                            .font_size(sec_font_size)
                            .color(theme.text_secondary()),
                        SizedBox::square((card_h * 0.095).round().max(4.0)),
                        Text::new(&display_text)
                            .font_size(primary_font_size)
                            .color(theme.text_primary()),
                    ]
                    .main_axis_alignment(MainAxisAlignment::Center)
                    .cross_axis_alignment(CrossAxisAlignment::End),
                ),
            SizedBox::square(col_gap),
            // 2. Keypad 5 Rows (Proportional & Centered)
            col![
                // Row 1: C, +/-, %, ÷
                row![
                    calc_btn(
                        "C",
                        btn_font_size,
                        &fn_style,
                        fn_text_color,
                        state.clone(),
                        |s| s.clear()
                    ),
                    SizedBox::square(col_gap),
                    calc_btn(
                        "+/-",
                        btn_font_size,
                        &fn_style,
                        fn_text_color,
                        state.clone(),
                        |s| s.toggle_sign()
                    ),
                    SizedBox::square(col_gap),
                    calc_btn(
                        "%",
                        btn_font_size,
                        &fn_style,
                        fn_text_color,
                        state.clone(),
                        |s| s.percent()
                    ),
                    SizedBox::square(col_gap),
                    calc_btn(
                        "÷",
                        btn_font_size,
                        &op_style,
                        op_text_color,
                        state.clone(),
                        |s| s.input_operator('÷')
                    ),
                ]
                .main_axis_alignment(MainAxisAlignment::Center),
                SizedBox::square(row_gap),
                // Row 2: 7, 8, 9, ×
                row![
                    calc_btn(
                        "7",
                        btn_font_size,
                        &num_style,
                        num_text_color,
                        state.clone(),
                        |s| s.input_digit('7')
                    ),
                    SizedBox::square(col_gap),
                    calc_btn(
                        "8",
                        btn_font_size,
                        &num_style,
                        num_text_color,
                        state.clone(),
                        |s| s.input_digit('8')
                    ),
                    SizedBox::square(col_gap),
                    calc_btn(
                        "9",
                        btn_font_size,
                        &num_style,
                        num_text_color,
                        state.clone(),
                        |s| s.input_digit('9')
                    ),
                    SizedBox::square(col_gap),
                    calc_btn(
                        "×",
                        btn_font_size,
                        &op_style,
                        op_text_color,
                        state.clone(),
                        |s| s.input_operator('×')
                    ),
                ]
                .main_axis_alignment(MainAxisAlignment::Center),
                SizedBox::square(row_gap),
                // Row 3: 4, 5, 6, -
                row![
                    calc_btn(
                        "4",
                        btn_font_size,
                        &num_style,
                        num_text_color,
                        state.clone(),
                        |s| s.input_digit('4')
                    ),
                    SizedBox::square(col_gap),
                    calc_btn(
                        "5",
                        btn_font_size,
                        &num_style,
                        num_text_color,
                        state.clone(),
                        |s| s.input_digit('5')
                    ),
                    SizedBox::square(col_gap),
                    calc_btn(
                        "6",
                        btn_font_size,
                        &num_style,
                        num_text_color,
                        state.clone(),
                        |s| s.input_digit('6')
                    ),
                    SizedBox::square(col_gap),
                    calc_btn(
                        "-",
                        btn_font_size,
                        &op_style,
                        op_text_color,
                        state.clone(),
                        |s| s.input_operator('-')
                    ),
                ]
                .main_axis_alignment(MainAxisAlignment::Center),
                SizedBox::square(row_gap),
                // Row 4: 1, 2, 3, +
                row![
                    calc_btn(
                        "1",
                        btn_font_size,
                        &num_style,
                        num_text_color,
                        state.clone(),
                        |s| s.input_digit('1')
                    ),
                    SizedBox::square(col_gap),
                    calc_btn(
                        "2",
                        btn_font_size,
                        &num_style,
                        num_text_color,
                        state.clone(),
                        |s| s.input_digit('2')
                    ),
                    SizedBox::square(col_gap),
                    calc_btn(
                        "3",
                        btn_font_size,
                        &num_style,
                        num_text_color,
                        state.clone(),
                        |s| s.input_digit('3')
                    ),
                    SizedBox::square(col_gap),
                    calc_btn(
                        "+",
                        btn_font_size,
                        &op_style,
                        op_text_color,
                        state.clone(),
                        |s| s.input_operator('+')
                    ),
                ]
                .main_axis_alignment(MainAxisAlignment::Center),
                SizedBox::square(row_gap),
                // Row 5: 0 (double width), ., =
                row![
                    calc_btn(
                        "0",
                        btn_font_size,
                        &zero_style,
                        num_text_color,
                        state.clone(),
                        |s| s.input_digit('0')
                    ),
                    SizedBox::square(col_gap),
                    calc_btn(
                        ".",
                        btn_font_size,
                        &num_style,
                        num_text_color,
                        state.clone(),
                        |s| s.input_dot()
                    ),
                    SizedBox::square(col_gap),
                    calc_btn(
                        "=",
                        btn_font_size,
                        &eq_style,
                        eq_text_color,
                        state.clone(),
                        |s| s.calculate()
                    ),
                ]
                .main_axis_alignment(MainAxisAlignment::Center),
            ]
            .main_axis_alignment(MainAxisAlignment::Center)
            .cross_axis_alignment(CrossAxisAlignment::Center),
        ]
        .main_axis_alignment(MainAxisAlignment::Center)
        .cross_axis_alignment(CrossAxisAlignment::Center),
    )
}
