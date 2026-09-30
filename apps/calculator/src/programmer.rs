use crate::expression::{evaluate, Token, MAX_TOKENS};
use crate::model::{closed_start, last_open};
use crate::{BinaryOp, CalcError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WordUnary {
    Not,
    Negate,
    ShiftLeft,
    ShiftRight,
    RotateLeft,
    RotateRight,
    Flip8,
    Flip16,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProgrammerState {
    pub value: u64,
    pub radix: u32,
    pub show_binary: bool,
    pub unicode: bool,
    pub error: Option<CalcError>,
    input: String,
    tokens: Vec<Token<u64>>,
    entry_active: bool,
    just_calculated: bool,
    repeated: Option<(BinaryOp, u64)>,
    history: String,
}
impl Default for ProgrammerState {
    fn default() -> Self {
        Self::new()
    }
}
impl ProgrammerState {
    pub(crate) fn valid_checkpoint(&self) -> bool {
        matches!(self.radix, 8 | 10 | 16)
            && self.input.len() <= 128
            && self.input.is_ascii()
            && self.history.len() <= 16_384
            && self.tokens.len() <= MAX_TOKENS
    }
    pub fn new() -> Self {
        Self {
            value: 0,
            radix: 16,
            show_binary: true,
            unicode: false,
            error: None,
            input: "0".into(),
            tokens: Vec::new(),
            entry_active: true,
            just_calculated: false,
            repeated: None,
            history: String::new(),
        }
    }
    pub fn clear(&mut self) {
        self.value = 0;
        self.input = "0".into();
        self.tokens.clear();
        self.error = None;
        self.entry_active = true;
        self.just_calculated = false;
        self.repeated = None;
        self.history.clear();
    }
    pub fn clear_label(&self) -> &'static str {
        if self.value == 0 || !self.entry_active || self.just_calculated || self.error.is_some() {
            "AC"
        } else {
            "C"
        }
    }
    pub fn clear_entry_or_all(&mut self) {
        if self.clear_label() == "AC" {
            self.clear();
        } else {
            self.set_value(0);
        }
    }
    fn fail(&mut self, e: CalcError) {
        self.error = Some(e);
        self.repeated = None;
    }
    fn formatted(&self, v: u64) -> String {
        match self.radix {
            8 => format!("{v:o}"),
            10 => format!("{}", v as i64),
            _ => format!("{v:X}"),
        }
    }
    pub fn display(&self) -> String {
        if self.error.is_some() {
            "Error".into()
        } else {
            self.formatted(self.value)
        }
    }
    pub fn set_radix(&mut self, radix: u32) {
        if matches!(radix, 8 | 10 | 16) {
            self.radix = radix;
            self.input = self.formatted(self.value);
        }
    }
    pub fn set_value(&mut self, v: u64) {
        self.value = v;
        self.input = self.formatted(v);
        self.entry_active = true;
        self.error = None;
    }
    pub fn digit_enabled(&self, c: char) -> bool {
        c.to_digit(self.radix).is_some()
    }
    pub fn input_digit(&mut self, c: char) {
        if !self.digit_enabled(c) {
            return;
        }
        if self.error.is_some() {
            self.clear();
        }
        if self.just_calculated {
            self.tokens.clear();
            self.history.clear();
            self.just_calculated = false;
            self.input = "0".into();
        }
        if !self.entry_active {
            if matches!(self.tokens.last(), Some(Token::Right | Token::Number(_))) {
                if let Err(e) = self.push(Token::Op(BinaryOp::Multiply)) {
                    self.fail(e);
                    return;
                }
            }
            self.input = "0".into();
            self.entry_active = true;
        }
        let mut next = if self.input == "0" {
            String::new()
        } else {
            self.input.clone()
        };
        next.push(c.to_ascii_uppercase());
        let value = if self.radix == 10 {
            next.parse::<i64>()
                .map(|v| v as u64)
                .map_err(|_| CalcError::Overflow)
        } else {
            u64::from_str_radix(&next, self.radix).map_err(|_| CalcError::Overflow)
        };
        match value {
            Ok(v) => {
                self.input = next;
                self.value = v;
            }
            Err(e) => self.fail(e),
        }
    }
    pub fn backspace(&mut self) {
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
                    self.set_value(0);
                }
                _ => {}
            }
            return;
        }
        self.input.pop();
        if self.input.is_empty() || self.input == "-" {
            self.input = "0".into();
        }
        self.value = if self.input.starts_with('-') {
            self.input.parse::<i64>().unwrap_or(0) as u64
        } else {
            u64::from_str_radix(&self.input, self.radix).unwrap_or(0)
        };
        self.just_calculated = false;
    }
    fn push(&mut self, t: Token<u64>) -> Result<(), CalcError> {
        if self.tokens.len() >= MAX_TOKENS {
            Err(CalcError::TooLong)
        } else {
            self.tokens.push(t);
            Ok(())
        }
    }
    fn commit(&mut self) -> Result<(), CalcError> {
        if self.entry_active {
            self.push(Token::Number(self.value))?;
            self.entry_active = false;
        }
        Ok(())
    }
    pub fn binary(&mut self, op: BinaryOp) {
        if self.error.is_some() {
            return;
        }
        self.just_calculated = false;
        self.repeated = None;
        let r = (|| {
            self.commit()?;
            if matches!(self.tokens.last(), Some(Token::Op(_))) {
                self.tokens.pop();
            }
            self.push(Token::Op(op))
        })();
        if let Err(e) = r {
            self.fail(e);
        }
    }
    pub fn open_parenthesis(&mut self) {
        if self.error.is_some() {
            return;
        }
        if self.just_calculated {
            self.clear();
        }
        let r = (|| {
            if self.entry_active && (self.value != 0 || !self.tokens.is_empty()) {
                self.commit()?;
            }
            if matches!(self.tokens.last(), Some(Token::Number(_) | Token::Right)) {
                self.push(Token::Op(BinaryOp::Multiply))?;
            }
            self.push(Token::Left)?;
            self.entry_active = false;
            Ok::<_, CalcError>(())
        })();
        if let Err(e) = r {
            self.fail(e);
        }
    }
    pub fn close_parenthesis(&mut self) {
        let r = (|| {
            self.commit()?;
            let i = last_open(&self.tokens).ok_or(CalcError::Syntax)?;
            let (v, _) = evaluate(&self.tokens[i + 1..], word_binary)?;
            self.push(Token::Right)?;
            self.value = v;
            self.input = self.formatted(v);
            self.entry_active = false;
            Ok::<_, CalcError>(())
        })();
        if let Err(e) = r {
            self.fail(e);
        }
    }
    pub fn calculate(&mut self) {
        if self.error.is_some() {
            return;
        }
        let r = (|| {
            if self.tokens.is_empty() {
                if let Some((op, rhs)) = self.repeated {
                    let v = word_binary(op, self.value, rhs)?;
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
            let (v, repeat) = evaluate(&self.tokens, word_binary)?;
            self.tokens.clear();
            self.repeated = repeat;
            self.set_value(v);
            self.just_calculated = true;
            self.history = format!("{expression} =");
            Ok::<_, CalcError>(())
        })();
        if let Err(e) = r {
            self.fail(e);
        }
    }
    pub fn unary(&mut self, op: WordUnary) {
        if self.error.is_some() {
            return;
        }
        let v = match op {
            WordUnary::Not => !self.value,
            WordUnary::Negate => self.value.wrapping_neg(),
            WordUnary::ShiftLeft => self.value << 1,
            WordUnary::ShiftRight => self.value >> 1,
            WordUnary::RotateLeft => self.value.rotate_left(1),
            WordUnary::RotateRight => self.value.rotate_right(1),
            // Flip adjacent 8-bit or 16-bit units in each 16-bit or 32-bit lane.
            WordUnary::Flip8 => {
                ((self.value & 0x00ff00ff00ff00ff) << 8) | ((self.value & 0xff00ff00ff00ff00) >> 8)
            }
            WordUnary::Flip16 => {
                ((self.value & 0x0000ffff0000ffff) << 16)
                    | ((self.value & 0xffff0000ffff0000) >> 16)
            }
        };
        self.set_value(v);
        self.just_calculated = false;
        self.repeated = None;
        self.replace_committed_operand(v);
    }
    fn replace_committed_operand(&mut self, v: u64) {
        // A unary operator replaces an already committed parenthesized operand.
        if matches!(self.tokens.last(), Some(Token::Right)) {
            if let Some(i) = closed_start(&self.tokens) {
                self.tokens.truncate(i);
                self.tokens.push(Token::Number(v));
                self.entry_active = false;
            }
        } else if matches!(self.tokens.last(), Some(Token::Number(_))) {
            self.tokens.pop();
            self.tokens.push(Token::Number(v));
            self.entry_active = false;
        }
    }
    pub fn toggle_bit(&mut self, bit: u32) {
        if bit < 64 {
            self.set_value(self.value ^ (1u64 << bit));
            self.just_calculated = false;
            self.repeated = None;
            self.replace_committed_operand(self.value);
        }
    }
    pub fn character_display(&self) -> String {
        if self.unicode {
            if self.value <= 0x10ffff {
                if let Some(c) = char::from_u32(self.value as u32) {
                    if !c.is_control() {
                        return format!("U+{:04X}  {c}", self.value);
                    }
                }
            }
            format!("U+{:04X}", self.value)
        } else if (32..=126).contains(&self.value) {
            format!("ASCII  {}", self.value as u8 as char)
        } else {
            format!("ASCII  0x{:02X}", self.value)
        }
    }
    pub fn expression_display(&self) -> String {
        if self.just_calculated {
            return self.history.clone();
        }
        let mut s = String::new();
        for t in &self.tokens {
            match t {
                Token::Number(v) => s.push_str(&self.formatted(*v)),
                Token::Op(op) => {
                    s.push(' ');
                    s.push_str(op.label());
                    s.push(' ');
                }
                Token::Left => s.push('('),
                Token::Right => s.push(')'),
            }
        }
        if self.entry_active {
            s.push_str(&self.formatted(self.value));
        }
        s
    }
}
fn word_binary(op: BinaryOp, a: u64, b: u64) -> Result<u64, CalcError> {
    Ok(match op {
        BinaryOp::Add => a.wrapping_add(b),
        BinaryOp::Subtract => a.wrapping_sub(b),
        BinaryOp::Multiply => a.wrapping_mul(b),
        BinaryOp::Divide => {
            if b == 0 {
                return Err(CalcError::DivideByZero);
            }
            (a as i64).wrapping_div(b as i64) as u64
        }
        BinaryOp::Mod => {
            if b == 0 {
                return Err(CalcError::DivideByZero);
            }
            (a as i64).wrapping_rem(b as i64) as u64
        }
        BinaryOp::And => a & b,
        BinaryOp::Or => a | b,
        BinaryOp::Xor => a ^ b,
        BinaryOp::Nor => !(a | b),
        BinaryOp::ShiftLeft => {
            if b >= 64 {
                0
            } else {
                a << b
            }
        }
        BinaryOp::ShiftRight => {
            if b >= 64 {
                0
            } else {
                a >> b
            }
        }
        _ => return Err(CalcError::Domain),
    })
}
