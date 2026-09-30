use crate::expression::{evaluate, finite, float_binary, Token, MAX_TOKENS};
use crate::{BinaryOp, CalcError, ProgrammerState};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CalcMode {
    Basic,
    Scientific,
    Programmer,
}
impl CalcMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Basic => "基本",
            Self::Scientific => "科学",
            Self::Programmer => "程序员",
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AngleUnit {
    Degrees,
    Radians,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnaryOp {
    Square,
    Cube,
    Reciprocal,
    Sqrt,
    Cbrt,
    Ln,
    Log10,
    Log2,
    Exp,
    Exp10,
    Exp2,
    Factorial,
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
    Sinh,
    Cosh,
    Tanh,
    Asinh,
    Acosh,
    Atanh,
}
impl UnaryOp {
    pub fn label(self) -> &'static str {
        match self {
            Self::Square => "square",
            Self::Cube => "cube",
            Self::Reciprocal => "1/x",
            Self::Sqrt => "sqrt",
            Self::Cbrt => "cbrt",
            Self::Ln => "ln",
            Self::Log10 => "log10",
            Self::Log2 => "log2",
            Self::Exp => "exp",
            Self::Exp10 => "10^x",
            Self::Exp2 => "2^x",
            Self::Factorial => "factorial",
            Self::Sin => "sin",
            Self::Cos => "cos",
            Self::Tan => "tan",
            Self::Asin => "asin",
            Self::Acos => "acos",
            Self::Atan => "atan",
            Self::Sinh => "sinh",
            Self::Cosh => "cosh",
            Self::Tanh => "tanh",
            Self::Asinh => "asinh",
            Self::Acosh => "acosh",
            Self::Atanh => "atanh",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CalcState {
    pub current_input: String,
    pub just_calculated: bool,
    pub secondary: String,
    pub mode: CalcMode,
    pub angle: AngleUnit,
    pub second_functions: bool,
    pub memory: f64,
    pub error: Option<CalcError>,
    pub programmer: ProgrammerState,
    tokens: Vec<Token<f64>>,
    entry_active: bool,
    exact: Option<f64>,
    pending_unary: Vec<UnaryOp>,
    group_unary: Vec<(usize, Vec<UnaryOp>)>,
    repeated: Option<(BinaryOp, f64, Option<f64>)>,
    percent_ratio: Option<f64>,
    rng: u64,
}
impl Default for CalcState {
    fn default() -> Self {
        Self::new()
    }
}
impl CalcState {
    /// Bound persisted input before it can enter the expression engine.
    pub fn valid_checkpoint(&self) -> bool {
        self.current_input.len() <= 128
            && self.current_input.is_ascii()
            && self.secondary.len() <= 16_384
            && self.memory.is_finite()
            && self.exact.is_none_or(f64::is_finite)
            && self.percent_ratio.is_none_or(f64::is_finite)
            && self.repeated.is_none_or(|(_, value, ratio)| {
                value.is_finite() && ratio.is_none_or(f64::is_finite)
            })
            && self.tokens.len() <= MAX_TOKENS
            && self
                .tokens
                .iter()
                .all(|t| !matches!(t, Token::Number(n) if !n.is_finite()))
            && self.pending_unary.len() <= 8
            && self.group_unary.len() <= MAX_TOKENS
            && self.group_unary.iter().all(|(i, ops)| {
                *i < self.tokens.len() && matches!(self.tokens[*i], Token::Left) && ops.len() <= 8
            })
            && self.programmer.valid_checkpoint()
    }
    pub fn new() -> Self {
        Self {
            current_input: "0".into(),
            just_calculated: false,
            secondary: String::new(),
            mode: CalcMode::Basic,
            angle: AngleUnit::Degrees,
            second_functions: false,
            memory: 0.0,
            error: None,
            programmer: ProgrammerState::new(),
            tokens: Vec::new(),
            entry_active: true,
            exact: None,
            pending_unary: Vec::new(),
            group_unary: Vec::new(),
            repeated: None,
            percent_ratio: None,
            rng: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(1)
                | 1,
        }
    }
    pub fn set_mode(&mut self, mode: CalcMode) {
        self.mode = mode;
    }
    fn fail(&mut self, e: CalcError) {
        self.error = Some(e);
        self.current_input = "Error".into();
        self.exact = None;
        self.repeated = None;
    }
    pub fn value(&self) -> Result<f64, CalcError> {
        self.error.map_or_else(
            || {
                self.exact
                    .map(Ok)
                    .unwrap_or_else(|| self.current_input.parse().map_err(|_| CalcError::Syntax))
                    .and_then(finite)
            },
            Err,
        )
    }
    fn set_value(&mut self, v: f64) {
        self.current_input = format_raw_number(v);
        self.exact = Some(v);
        self.entry_active = true;
        self.error = None;
    }
    fn reset_expression(&mut self) {
        self.tokens.clear();
        self.pending_unary.clear();
        self.group_unary.clear();
        self.secondary.clear();
        self.percent_ratio = None;
    }
    pub fn clear(&mut self) {
        if self.mode == CalcMode::Programmer {
            self.programmer.clear();
            return;
        }
        self.reset_expression();
        self.current_input = "0".into();
        self.exact = None;
        self.error = None;
        self.entry_active = true;
        self.just_calculated = false;
        self.repeated = None;
    }
    pub fn clear_label(&self) -> &'static str {
        if self.mode == CalcMode::Programmer {
            return self.programmer.clear_label();
        }
        if self.current_input == "0"
            || self.just_calculated
            || self.error.is_some()
            || !self.entry_active
        {
            "AC"
        } else {
            "C"
        }
    }
    pub fn clear_entry_or_all(&mut self) {
        if self.mode == CalcMode::Programmer {
            self.programmer.clear_entry_or_all();
            return;
        }
        if self.clear_label() == "AC" {
            self.clear();
        } else {
            self.current_input = "0".into();
            self.exact = None;
            self.pending_unary.clear();
            self.percent_ratio = None;
        }
    }
    pub fn input_digit(&mut self, digit: char) {
        if self.mode == CalcMode::Programmer {
            self.programmer.input_digit(digit);
            return;
        }
        if !digit.is_ascii_digit() {
            return;
        }
        if self.error.is_some() {
            self.clear();
        }
        if self.just_calculated {
            self.reset_expression();
            self.current_input = "0".into();
            self.just_calculated = false;
        }
        if !self.entry_active {
            if matches!(self.tokens.last(), Some(Token::Right | Token::Number(_))) {
                if let Err(e) = self.push(Token::Op(BinaryOp::Multiply)) {
                    self.fail(e);
                    return;
                }
            }
            self.current_input = "0".into();
            self.entry_active = true;
        }
        if self.current_input == "0" {
            self.current_input = digit.to_string();
        } else if self.current_input.len() < 24 {
            self.current_input.push(digit);
        }
        self.exact = None;
        self.percent_ratio = None;
    }
    pub fn input_dot(&mut self) {
        if self.mode == CalcMode::Programmer {
            return;
        }
        if self.error.is_some() {
            self.clear();
        }
        if self.just_calculated {
            self.reset_expression();
            self.current_input = "0".into();
            self.just_calculated = false;
        }
        if !self.entry_active {
            if matches!(self.tokens.last(), Some(Token::Right | Token::Number(_))) {
                if let Err(e) = self.push(Token::Op(BinaryOp::Multiply)) {
                    self.fail(e);
                    return;
                }
            }
            self.current_input = "0".into();
            self.entry_active = true;
        }
        if !self.current_input.contains('.') && !self.current_input.contains('e') {
            self.current_input.push('.');
            self.exact = None;
        }
    }
    pub fn input_exponent(&mut self) {
        if self.mode != CalcMode::Programmer
            && self.entry_active
            && !self.current_input.contains('e')
            && self.value().is_ok()
        {
            self.current_input.push('e');
            self.exact = None;
            self.just_calculated = false;
            self.repeated = None;
        }
    }
    pub fn backspace(&mut self) {
        if self.mode == CalcMode::Programmer {
            self.programmer.backspace();
            return;
        }
        if self.error.is_some() {
            self.clear();
            return;
        }
        if !self.entry_active {
            if matches!(self.tokens.last(), Some(Token::Op(_))) {
                self.tokens.pop();
            }
            match self.tokens.last().copied() {
                Some(Token::Number(v)) => {
                    self.tokens.pop();
                    self.set_value(v);
                }
                Some(Token::Right) => {
                    self.tokens.pop();
                    if let Some(Token::Number(v)) = self.tokens.last().copied() {
                        self.tokens.pop();
                        self.set_value(v);
                    }
                }
                Some(Token::Left) => {
                    self.tokens.pop();
                    self.group_unary.retain(|(i, _)| *i < self.tokens.len());
                    self.set_value(0.0);
                }
                _ => {}
            }
            return;
        }
        if self.just_calculated {
            self.reset_expression();
            self.just_calculated = false;
        }
        self.current_input.pop();
        if self.current_input.is_empty() || self.current_input == "-" {
            self.current_input = "0".into();
        }
        self.exact = None;
        self.percent_ratio = None;
    }
    fn push(&mut self, t: Token<f64>) -> Result<(), CalcError> {
        if self.tokens.len() >= MAX_TOKENS {
            Err(CalcError::TooLong)
        } else {
            self.tokens.push(t);
            Ok(())
        }
    }
    fn commit(&mut self) -> Result<(), CalcError> {
        if self.entry_active {
            let mut v = self.value()?;
            for op in self.pending_unary.iter().rev() {
                v = unary(*op, v, self.angle)?;
            }
            self.pending_unary.clear();
            self.set_value(v);
            self.push(Token::Number(v))?;
            self.entry_active = false;
        }
        Ok(())
    }
    pub fn input_operator(&mut self, op: char) {
        let op = match op {
            '+' => BinaryOp::Add,
            '-' | '−' => BinaryOp::Subtract,
            '×' | '*' => BinaryOp::Multiply,
            '÷' | '/' => BinaryOp::Divide,
            _ => return,
        };
        self.binary(op);
    }
    pub fn binary(&mut self, op: BinaryOp) {
        if self.mode == CalcMode::Programmer {
            self.programmer.binary(op);
            return;
        }
        if self.error.is_some() {
            return;
        }
        self.just_calculated = false;
        self.repeated = None;
        self.percent_ratio = None;
        let result = (|| {
            self.commit()?;
            if matches!(self.tokens.last(), Some(Token::Op(_))) {
                self.tokens.pop();
            }
            self.push(Token::Op(op))
        })();
        if let Err(e) = result {
            self.fail(e);
        }
    }
    pub fn open_parenthesis(&mut self) {
        if self.mode == CalcMode::Programmer {
            self.programmer.open_parenthesis();
            return;
        }
        if self.error.is_some() {
            return;
        }
        if self.just_calculated {
            self.clear();
        }
        let r = (|| {
            let functions = std::mem::take(&mut self.pending_unary);
            if functions.is_empty()
                && self.entry_active
                && (self.current_input != "0" || !self.tokens.is_empty())
            {
                self.commit()?;
            }
            if matches!(self.tokens.last(), Some(Token::Number(_) | Token::Right)) {
                self.push(Token::Op(BinaryOp::Multiply))?;
            }
            let start = self.tokens.len();
            self.push(Token::Left)?;
            if !functions.is_empty() {
                self.group_unary.push((start, functions));
            }
            self.entry_active = false;
            self.current_input = "0".into();
            self.exact = None;
            Ok::<_, CalcError>(())
        })();
        if let Err(e) = r {
            self.fail(e);
        }
    }
    pub fn close_parenthesis(&mut self) {
        if self.mode == CalcMode::Programmer {
            self.programmer.close_parenthesis();
            return;
        }
        let r = (|| {
            self.commit()?;
            let start = last_open(&self.tokens).ok_or(CalcError::Syntax)?;
            let (mut v, _) = evaluate(&self.tokens[start + 1..], float_binary)?;
            if self.group_unary.last().is_some_and(|(i, _)| *i == start) {
                let (_, functions) = self.group_unary.pop().unwrap();
                for op in functions.into_iter().rev() {
                    v = unary(op, v, self.angle)?;
                }
                self.tokens.truncate(start);
                self.push(Token::Number(v))?;
            } else {
                self.push(Token::Right)?;
            }
            self.current_input = format_raw_number(v);
            self.exact = Some(v);
            self.entry_active = false;
            Ok::<_, CalcError>(())
        })();
        if let Err(e) = r {
            self.fail(e);
        }
    }
    pub fn calculate(&mut self) {
        if self.mode == CalcMode::Programmer {
            self.programmer.calculate();
            return;
        }
        let r = (|| {
            if self.tokens.is_empty() && self.pending_unary.is_empty() {
                if let Some((op, rhs, percentage)) = self.repeated {
                    let lhs = self.value()?;
                    let rhs = percentage.map_or(rhs, |p| lhs * p);
                    let v = float_binary(op, lhs, rhs)?;
                    self.secondary = format!(
                        "{} {} {} =",
                        format_raw_number(lhs),
                        op.label(),
                        format_raw_number(rhs)
                    );
                    self.set_value(v);
                    self.just_calculated = true;
                    return Ok(());
                }
            }
            if !self.entry_active && matches!(self.tokens.last(), Some(Token::Op(_))) {
                self.entry_active = true;
            }
            let expression = self.expression_display();
            self.commit()?;
            let (v, repeat) = evaluate(&self.tokens, float_binary)?;
            self.repeated = repeat.map(|(op, rhs)| {
                (
                    op,
                    rhs,
                    self.percent_ratio
                        .filter(|_| matches!(op, BinaryOp::Add | BinaryOp::Subtract)),
                )
            });
            self.tokens.clear();
            self.pending_unary.clear();
            self.set_value(v);
            self.just_calculated = true;
            self.secondary = format!("{expression} =");
            self.percent_ratio = None;
            Ok::<_, CalcError>(())
        })();
        if let Err(e) = r {
            self.fail(e);
        }
    }
    pub fn toggle_sign(&mut self) {
        if self.mode == CalcMode::Programmer {
            self.programmer.unary(crate::WordUnary::Negate);
            return;
        }
        if self.error.is_some() {
            return;
        }
        if let Some(i) = self.current_input.find('e') {
            if self.current_input[i + 1..].starts_with('-') {
                self.current_input.remove(i + 1);
            } else {
                self.current_input.insert(i + 1, '-');
            }
            self.exact = None;
        } else if let Ok(v) = self.value() {
            self.replace_operand(-v);
        }
        self.repeated = None;
        self.just_calculated = false;
    }
    pub fn percent(&mut self) {
        let r = (|| {
            let p = self.value()? / 100.0;
            if matches!(
                self.tokens.last(),
                Some(Token::Op(BinaryOp::Add | BinaryOp::Subtract))
            ) {
                let start = last_open(&self.tokens).map_or(0, |i| i + 1);
                let (lhs, _) = evaluate(&self.tokens[start..self.tokens.len() - 1], float_binary)?;
                self.replace_operand(lhs * p);
                self.percent_ratio = Some(p);
            } else {
                self.replace_operand(p);
                self.percent_ratio = None;
            }
            Ok::<_, CalcError>(())
        })();
        if let Err(e) = r {
            self.fail(e);
        }
    }
    pub fn function(&mut self, op: UnaryOp) {
        if self.error.is_some() {
            return;
        }
        self.repeated = None;
        if (!self.entry_active
            && !matches!(self.tokens.last(), Some(Token::Right | Token::Number(_))))
            || (self.current_input == "0" && self.tokens.is_empty() && !self.just_calculated)
        {
            if self.pending_unary.len() >= 8 {
                self.fail(CalcError::TooLong);
            } else {
                self.pending_unary.push(op);
                self.current_input = "0".into();
                self.exact = None;
                self.entry_active = true;
            }
            return;
        }
        match self.value().and_then(|v| unary(op, v, self.angle)) {
            Ok(v) => {
                self.replace_operand(v);
                self.just_calculated = false;
            }
            Err(e) => self.fail(e),
        }
    }
    pub fn constant(&mut self, v: f64) {
        if let Err(e) = finite(v) {
            self.fail(e);
            return;
        }
        if self.just_calculated {
            self.reset_expression();
        }
        if !self.entry_active && matches!(self.tokens.last(), Some(Token::Right | Token::Number(_)))
        {
            if let Err(e) = self.push(Token::Op(BinaryOp::Multiply)) {
                self.fail(e);
                return;
            }
        }
        self.set_value(v);
        self.repeated = None;
        self.just_calculated = false;
    }
    pub fn random(&mut self) {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        self.constant((self.rng >> 11) as f64 / (1u64 << 53) as f64);
    }
    pub fn memory_add(&mut self, subtract: bool) {
        match self
            .value()
            .and_then(|v| finite(self.memory + if subtract { -v } else { v }))
        {
            Ok(v) => self.memory = v,
            Err(e) => self.fail(e),
        }
    }
    pub fn memory_recall(&mut self) {
        self.constant(self.memory);
    }
    pub fn primary_display(&self) -> String {
        if self.mode == CalcMode::Programmer {
            self.programmer.display()
        } else {
            add_commas(&self.current_input)
        }
    }
    pub fn active_error(&self) -> Option<CalcError> {
        if self.mode == CalcMode::Programmer {
            self.programmer.error
        } else {
            self.error
        }
    }
    pub fn secondary_display(&self) -> &str {
        &self.secondary
    }
    fn replace_operand(&mut self, v: f64) {
        let start = if !self.entry_active {
            match self.tokens.last() {
                Some(Token::Number(_)) => Some(self.tokens.len() - 1),
                Some(Token::Right) => closed_start(&self.tokens),
                _ => None,
            }
        } else {
            None
        };
        self.set_value(v);
        if let Some(i) = start {
            self.tokens.truncate(i);
            self.tokens.push(Token::Number(v));
            self.entry_active = false;
        }
    }
    pub fn expression_display(&self) -> String {
        if self.mode == CalcMode::Programmer {
            return self.programmer.expression_display();
        }
        if self.just_calculated {
            return self.secondary.clone();
        }
        let mut s = String::new();
        for t in &self.tokens {
            match t {
                Token::Number(v) => s.push_str(&format_raw_number(*v)),
                Token::Op(op) => {
                    s.push(' ');
                    s.push_str(op.label());
                    s.push(' ');
                }
                Token::Left => s.push('('),
                Token::Right => s.push(')'),
            }
        }
        for op in &self.pending_unary {
            s.push_str(op.label());
            s.push('(');
        }
        if self.entry_active {
            s.push_str(&self.current_input);
        }
        for _ in &self.pending_unary {
            s.push(')');
        }
        s
    }
}
pub(crate) fn last_open<T>(tokens: &[Token<T>]) -> Option<usize> {
    let mut depth = 0;
    for (i, t) in tokens.iter().enumerate().rev() {
        match t {
            Token::Right => depth += 1,
            Token::Left => {
                if depth == 0 {
                    return Some(i);
                }
                depth -= 1;
            }
            _ => {}
        }
    }
    None
}
pub(crate) fn closed_start<T>(tokens: &[Token<T>]) -> Option<usize> {
    let mut depth = 0;
    for (i, token) in tokens.iter().enumerate().rev() {
        match token {
            Token::Right => depth += 1,
            Token::Left => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}
fn unary(op: UnaryOp, v: f64, angle: AngleUnit) -> Result<f64, CalcError> {
    let radians = if angle == AngleUnit::Degrees {
        v.to_radians()
    } else {
        v
    };
    let inverse = |v: f64| {
        if angle == AngleUnit::Degrees {
            v.to_degrees()
        } else {
            v
        }
    };
    let value = match op {
        UnaryOp::Square => v * v,
        UnaryOp::Cube => v * v * v,
        UnaryOp::Reciprocal => {
            if v == 0.0 {
                return Err(CalcError::DivideByZero);
            }
            1.0 / v
        }
        UnaryOp::Sqrt => v.sqrt(),
        UnaryOp::Cbrt => v.cbrt(),
        UnaryOp::Ln => {
            if v <= 0.0 {
                return Err(CalcError::Domain);
            }
            v.ln()
        }
        UnaryOp::Log10 => {
            if v <= 0.0 {
                return Err(CalcError::Domain);
            }
            v.log10()
        }
        UnaryOp::Log2 => {
            if v <= 0.0 {
                return Err(CalcError::Domain);
            }
            v.log2()
        }
        UnaryOp::Exp => v.exp(),
        UnaryOp::Exp10 => 10.0f64.powf(v),
        UnaryOp::Exp2 => 2.0f64.powf(v),
        UnaryOp::Factorial => {
            if v < 0.0 || v.fract() != 0.0 {
                return Err(CalcError::Domain);
            }
            if v > 170.0 {
                return Err(CalcError::Overflow);
            }
            (1..=v as u32).fold(1.0, |n, i| n * i as f64)
        }
        UnaryOp::Sin => radians.sin(),
        UnaryOp::Cos => radians.cos(),
        UnaryOp::Tan => {
            if radians.cos().abs() < 1e-15 {
                return Err(CalcError::Domain);
            }
            radians.tan()
        }
        UnaryOp::Asin => inverse(v.asin()),
        UnaryOp::Acos => inverse(v.acos()),
        UnaryOp::Atan => inverse(v.atan()),
        UnaryOp::Sinh => v.sinh(),
        UnaryOp::Cosh => v.cosh(),
        UnaryOp::Tanh => v.tanh(),
        UnaryOp::Asinh => v.asinh(),
        UnaryOp::Acosh => v.acosh(),
        UnaryOp::Atanh => v.atanh(),
    };
    finite(
        if matches!(op, UnaryOp::Sin | UnaryOp::Cos) && value.abs() < 1e-15 {
            0.0
        } else {
            value
        },
    )
}
pub fn eval_op(a: f64, op: char, b: f64) -> f64 {
    let op = match op {
        '+' => BinaryOp::Add,
        '-' | '−' => BinaryOp::Subtract,
        '×' | '*' => BinaryOp::Multiply,
        '÷' | '/' => BinaryOp::Divide,
        _ => return b,
    };
    float_binary(op, a, b).unwrap_or(f64::NAN)
}
pub fn format_raw_number(n: f64) -> String {
    if !n.is_finite() {
        return "Error".into();
    }
    if n == 0.0 {
        return "0".into();
    }
    if n.abs() >= 1e12 || n.abs() < 1e-9 {
        let raw = format!("{n:.11e}");
        let (m, e) = raw.split_once('e').unwrap();
        return format!(
            "{}e{}",
            m.trim_end_matches('0').trim_end_matches('.'),
            e.parse::<i32>().unwrap()
        );
    }
    let decimals = (11.0 - n.abs().log10().floor()).clamp(0.0, 18.0) as usize;
    let raw = format!("{n:.decimals$}");
    if raw.contains('.') {
        raw.trim_end_matches('0').trim_end_matches('.').into()
    } else {
        raw
    }
}
pub fn add_commas(s: &str) -> String {
    if s == "Error" || s.contains('e') {
        return s.into();
    }
    let (sign, s) = s.strip_prefix('-').map_or(("", s), |v| ("-", v));
    let (integer, fraction) = s.split_once('.').map_or((s, None), |(i, f)| (i, Some(f)));
    let mut out = sign.to_owned();
    for (i, c) in integer.chars().enumerate() {
        if i > 0 && (integer.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    if let Some(f) = fraction {
        out.push('.');
        out.push_str(f);
    }
    out
}
