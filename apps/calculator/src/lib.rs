//! Pad calculator, implemented in the original tiny-flutter Rust UI.
mod expression;
mod model;
mod programmer;
mod ui;

pub use expression::{BinaryOp, CalcError};
pub use model::{add_commas, eval_op, format_raw_number, AngleUnit, CalcMode, CalcState, UnaryOp};
pub use programmer::{ProgrammerState, WordUnary};
pub use ui::{
    build_calculator_ui, build_mode_selector, calculator_layout, mode_selector_layout,
    CalculatorKey, CalculatorLayout, KeyAction, CALCULATOR_BG,
};
