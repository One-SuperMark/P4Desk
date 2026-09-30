use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CalcError {
    DivideByZero,
    Domain,
    Overflow,
    Syntax,
    TooLong,
}
impl CalcError {
    pub fn message(self) -> &'static str {
        match self {
            Self::DivideByZero => "不能除以零",
            Self::Domain => "无效输入",
            Self::Overflow => "超出范围",
            Self::Syntax => "请检查表达式",
            Self::TooLong => "表达式过长",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Power,
    Root,
    Mod,
    And,
    Or,
    Xor,
    Nor,
    ShiftLeft,
    ShiftRight,
}
impl BinaryOp {
    fn precedence(self) -> u8 {
        match self {
            Self::Or | Self::Nor => 0,
            Self::Xor => 1,
            Self::And => 2,
            Self::ShiftLeft | Self::ShiftRight => 3,
            Self::Add | Self::Subtract => 4,
            Self::Multiply | Self::Divide | Self::Mod => 5,
            Self::Power | Self::Root => 6,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Add => "+",
            Self::Subtract => "−",
            Self::Multiply => "×",
            Self::Divide => "÷",
            Self::Power => "^",
            Self::Root => "root",
            Self::Mod => "mod",
            Self::And => "AND",
            Self::Or => "OR",
            Self::Xor => "XOR",
            Self::Nor => "NOR",
            Self::ShiftLeft => "<<",
            Self::ShiftRight => ">>",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub(crate) enum Token<T> {
    Number(T),
    Op(BinaryOp),
    Left,
    Right,
}
pub(crate) const MAX_TOKENS: usize = 128;

/// Bounded shunting-yard evaluation. The last reduction is the root operation for repeated equals.
pub(crate) fn evaluate<T: Copy>(
    tokens: &[Token<T>],
    binary: impl Fn(BinaryOp, T, T) -> Result<T, CalcError>,
) -> Result<(T, Option<(BinaryOp, T)>), CalcError> {
    if tokens.is_empty() {
        return Err(CalcError::Syntax);
    }
    if tokens.len() > MAX_TOKENS {
        return Err(CalcError::TooLong);
    }
    let mut values = Vec::new();
    let mut ops: Vec<Token<T>> = Vec::new();
    let mut expect_value = true;
    let mut repeat = None;
    let reduce = |values: &mut Vec<T>, op: BinaryOp, repeat: &mut Option<(BinaryOp, T)>| {
        let b = values.pop().ok_or(CalcError::Syntax)?;
        let a = values.pop().ok_or(CalcError::Syntax)?;
        values.push(binary(op, a, b)?);
        *repeat = Some((op, b));
        Ok::<_, CalcError>(())
    };
    for token in tokens {
        match *token {
            Token::Number(v) if expect_value => {
                values.push(v);
                expect_value = false;
            }
            Token::Left if expect_value => ops.push(Token::Left),
            Token::Right if !expect_value => {
                let mut found = false;
                while let Some(op) = ops.pop() {
                    match op {
                        Token::Left => {
                            found = true;
                            break;
                        }
                        Token::Op(op) => reduce(&mut values, op, &mut repeat)?,
                        _ => unreachable!(),
                    }
                }
                if !found {
                    return Err(CalcError::Syntax);
                }
            }
            Token::Op(next) if !expect_value => {
                while let Some(Token::Op(top)) = ops.last().copied() {
                    if top.precedence() < next.precedence()
                        || (top.precedence() == next.precedence()
                            && matches!(next, BinaryOp::Power | BinaryOp::Root))
                    {
                        break;
                    }
                    ops.pop();
                    reduce(&mut values, top, &mut repeat)?;
                }
                ops.push(Token::Op(next));
                expect_value = true;
            }
            _ => return Err(CalcError::Syntax),
        }
    }
    if expect_value {
        return Err(CalcError::Syntax);
    }
    while let Some(op) = ops.pop() {
        if let Token::Op(op) = op {
            reduce(&mut values, op, &mut repeat)?;
        } else {
            return Err(CalcError::Syntax);
        }
    }
    if values.len() != 1 {
        return Err(CalcError::Syntax);
    }
    Ok((values[0], repeat))
}

pub(crate) fn float_binary(op: BinaryOp, a: f64, b: f64) -> Result<f64, CalcError> {
    let v = match op {
        BinaryOp::Add => a + b,
        BinaryOp::Subtract => a - b,
        BinaryOp::Multiply => a * b,
        BinaryOp::Divide => {
            if b == 0.0 {
                return Err(CalcError::DivideByZero);
            }
            a / b
        }
        BinaryOp::Power => a.powf(b),
        BinaryOp::Root => {
            if b == 0.0 {
                return Err(CalcError::Domain);
            }
            if a < 0.0 && b.fract() == 0.0 && b.abs() % 2.0 == 1.0 {
                -(-a).powf(1.0 / b)
            } else {
                a.powf(1.0 / b)
            }
        }
        _ => return Err(CalcError::Domain),
    };
    finite(v)
}
pub(crate) fn finite(v: f64) -> Result<f64, CalcError> {
    if v.is_nan() {
        Err(CalcError::Domain)
    } else if !v.is_finite() {
        Err(CalcError::Overflow)
    } else {
        Ok(v)
    }
}
