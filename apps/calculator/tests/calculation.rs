use calculator::{
    AngleUnit, BinaryOp as B, CalcError, CalcMode, CalcState, KeyAction, ProgrammerState,
    UnaryOp as U, WordUnary as W,
};

fn digits(s: &mut CalcState, input: &str) {
    for c in input.chars() {
        if c == '.' {
            s.input_dot();
        } else {
            s.input_digit(c);
        }
    }
}
fn number(s: &mut CalcState, input: &str) {
    s.clear();
    digits(s, input);
}
fn close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 1e-12 * expected.abs().max(1.0),
        "{actual} != {expected}"
    );
}
fn word(input: &str, radix: u32) -> ProgrammerState {
    let mut s = ProgrammerState::new();
    s.set_radix(radix);
    for c in input.chars() {
        s.input_digit(c);
    }
    s
}

#[test]
fn precedence_and_nested_parentheses() {
    let mut s = CalcState::new();
    digits(&mut s, "2");
    s.binary(B::Add);
    digits(&mut s, "3");
    s.binary(B::Multiply);
    digits(&mut s, "4");
    s.calculate();
    assert_eq!(s.value(), Ok(14.0));
    s.clear();
    s.open_parenthesis();
    digits(&mut s, "2");
    s.binary(B::Add);
    digits(&mut s, "3");
    s.close_parenthesis();
    s.binary(B::Multiply);
    s.open_parenthesis();
    digits(&mut s, "4");
    s.binary(B::Subtract);
    digits(&mut s, "1");
    s.close_parenthesis();
    s.calculate();
    assert_eq!(s.value(), Ok(15.0));
}
#[test]
fn implicit_multiplication_after_a_group() {
    let mut s = CalcState::new();
    s.open_parenthesis();
    digits(&mut s, "2");
    s.binary(B::Add);
    digits(&mut s, "3");
    s.close_parenthesis();
    digits(&mut s, "4");
    s.calculate();
    assert_eq!(s.value(), Ok(20.0));
}
#[test]
fn powers_are_right_associative_and_roots_handle_negative_odd_degree() {
    let mut s = CalcState::new();
    digits(&mut s, "2");
    s.binary(B::Power);
    digits(&mut s, "3");
    s.binary(B::Power);
    digits(&mut s, "2");
    s.calculate();
    assert_eq!(s.value(), Ok(512.0));
    number(&mut s, "8");
    s.toggle_sign();
    s.binary(B::Root);
    digits(&mut s, "3");
    s.calculate();
    close(s.value().unwrap(), -2.0);
}
#[test]
fn division_by_zero_reports_error_and_next_digit_recovers() {
    let mut s = CalcState::new();
    digits(&mut s, "8");
    s.binary(B::Divide);
    digits(&mut s, "0");
    s.calculate();
    assert_eq!(s.active_error(), Some(CalcError::DivideByZero));
    digits(&mut s, "7");
    assert_eq!(s.value(), Ok(7.0));
    assert!(s.active_error().is_none());
}
#[test]
fn unfinished_group_and_incomplete_exponent_report_syntax() {
    let mut s = CalcState::new();
    s.open_parenthesis();
    digits(&mut s, "3");
    s.calculate();
    assert_eq!(s.active_error(), Some(CalcError::Syntax));
    s.clear();
    digits(&mut s, "1");
    s.input_exponent();
    s.calculate();
    assert_eq!(s.active_error(), Some(CalcError::Syntax));
}
#[test]
fn repeat_equals_applies_root_operation_to_result_or_new_operand() {
    let mut s = CalcState::new();
    digits(&mut s, "10");
    s.binary(B::Add);
    digits(&mut s, "2");
    s.calculate();
    s.calculate();
    assert_eq!(s.value(), Ok(14.0));
    digits(&mut s, "30");
    s.calculate();
    assert_eq!(s.value(), Ok(32.0));
}
#[test]
fn additive_percentage_repeat_uses_ratio_for_new_operand() {
    let mut s = CalcState::new();
    digits(&mut s, "100");
    s.binary(B::Add);
    digits(&mut s, "15");
    s.percent();
    s.calculate();
    assert_eq!(s.value(), Ok(115.0));
    digits(&mut s, "150");
    s.calculate();
    assert_eq!(s.value(), Ok(172.5));
    s.calculate();
    close(s.value().unwrap(), 198.375);
}
#[test]
fn percentage_multiplication_and_standalone() {
    let mut s = CalcState::new();
    digits(&mut s, "200");
    s.binary(B::Multiply);
    digits(&mut s, "10");
    s.percent();
    s.calculate();
    assert_eq!(s.value(), Ok(20.0));
    number(&mut s, "25");
    s.percent();
    assert_eq!(s.value(), Ok(0.25));
}
#[test]
fn c_clears_operand_ac_clears_expression_and_operator_can_be_replaced() {
    let mut s = CalcState::new();
    digits(&mut s, "8");
    s.binary(B::Add);
    s.binary(B::Multiply);
    digits(&mut s, "99");
    assert_eq!(s.clear_label(), "C");
    s.clear_entry_or_all();
    digits(&mut s, "2");
    s.calculate();
    assert_eq!(s.value(), Ok(16.0));
    assert_eq!(s.clear_label(), "AC");
    s.clear_entry_or_all();
    assert_eq!(s.value(), Ok(0.0));
}
#[test]
fn backspace_edits_digit_pending_operator_and_group_without_losing_prefix() {
    let mut s = CalcState::new();
    digits(&mut s, "123");
    s.backspace();
    assert_eq!(s.value(), Ok(12.0));
    s.binary(B::Add);
    s.backspace();
    digits(&mut s, "3");
    assert_eq!(s.value(), Ok(123.0));
    s.clear();
    s.open_parenthesis();
    digits(&mut s, "12");
    s.close_parenthesis();
    s.backspace();
    s.backspace();
    digits(&mut s, "3");
    s.close_parenthesis();
    s.calculate();
    assert_eq!(s.value(), Ok(13.0));
}
#[test]
fn result_display_rounding_does_not_reduce_internal_precision() {
    let mut s = CalcState::new();
    digits(&mut s, "1");
    s.binary(B::Divide);
    digits(&mut s, "3");
    s.calculate();
    assert_eq!(s.current_input, "0.333333333333");
    s.binary(B::Multiply);
    digits(&mut s, "3");
    s.calculate();
    assert_eq!(s.value(), Ok(1.0));
}
#[test]
fn ee_supports_negative_exponent_and_backspace_repair() {
    let mut s = CalcState::new();
    digits(&mut s, "2.5");
    s.input_exponent();
    s.toggle_sign();
    digits(&mut s, "3");
    s.calculate();
    close(s.value().unwrap(), 0.0025);
    number(&mut s, "2");
    s.input_exponent();
    s.backspace();
    digits(&mut s, "3");
    assert_eq!(s.value(), Ok(23.0));
}
#[test]
fn degrees_radians_and_inverse_trig() {
    let mut s = CalcState::new();
    number(&mut s, "30");
    s.function(U::Sin);
    close(s.value().unwrap(), 0.5);
    s.function(U::Asin);
    close(s.value().unwrap(), 30.0);
    s.angle = AngleUnit::Radians;
    s.constant(std::f64::consts::FRAC_PI_2);
    s.function(U::Sin);
    close(s.value().unwrap(), 1.0);
}
#[test]
fn prefix_function_can_apply_to_an_entire_group() {
    let mut s = CalcState::new();
    s.function(U::Sin);
    s.open_parenthesis();
    digits(&mut s, "30");
    s.binary(B::Add);
    digits(&mut s, "60");
    s.close_parenthesis();
    s.calculate();
    close(s.value().unwrap(), 1.0);
}
#[test]
fn postfix_functions_and_sign_replace_closed_group() {
    let mut s = CalcState::new();
    s.open_parenthesis();
    digits(&mut s, "3");
    s.binary(B::Add);
    digits(&mut s, "1");
    s.close_parenthesis();
    s.function(U::Square);
    s.function(U::Sqrt);
    s.toggle_sign();
    s.calculate();
    assert_eq!(s.value(), Ok(-4.0));
}
#[test]
fn unary_after_result_does_not_repeat_previous_binary_operation() {
    let mut s = CalcState::new();
    digits(&mut s, "3");
    s.binary(B::Add);
    digits(&mut s, "1");
    s.calculate();
    s.function(U::Square);
    s.calculate();
    assert_eq!(s.value(), Ok(16.0));
}
#[test]
fn scientific_functions_and_domain_limits() {
    for (input, op, expected) in [
        ("3", U::Cube, 27.0),
        ("4", U::Reciprocal, 0.25),
        ("27", U::Cbrt, 3.0),
        ("1000", U::Log10, 3.0),
        ("8", U::Log2, 3.0),
        ("2", U::Exp10, 100.0),
        ("3", U::Exp2, 8.0),
        ("5", U::Factorial, 120.0),
    ] {
        let mut s = CalcState::new();
        digits(&mut s, input);
        s.function(op);
        close(s.value().unwrap(), expected);
    }
    for (input, op, error) in [
        ("0", U::Reciprocal, CalcError::DivideByZero),
        ("0", U::Ln, CalcError::Domain),
        ("2", U::Asin, CalcError::Domain),
        ("90", U::Tan, CalcError::Domain),
        ("171", U::Factorial, CalcError::Overflow),
        ("2.5", U::Factorial, CalcError::Domain),
    ] {
        let mut s = CalcState::new();
        digits(&mut s, input);
        s.function(op);
        s.calculate();
        assert_eq!(s.active_error(), Some(error));
    }
}
#[test]
fn hyperbolic_inverse_and_random_range() {
    let mut s = CalcState::new();
    number(&mut s, "2");
    s.function(U::Sinh);
    s.function(U::Asinh);
    close(s.value().unwrap(), 2.0);
    for _ in 0..100 {
        s.random();
        assert!((0.0..1.0).contains(&s.value().unwrap()));
    }
}
#[test]
fn memory_and_mode_switch_keep_independent_contexts() {
    let mut s = CalcState::new();
    digits(&mut s, "12");
    s.memory_add(false);
    number(&mut s, "5");
    s.memory_add(true);
    s.clear();
    s.memory_recall();
    assert_eq!(s.value(), Ok(7.0));
    s.set_mode(CalcMode::Scientific);
    s.angle = AngleUnit::Radians;
    s.second_functions = true;
    s.set_mode(CalcMode::Programmer);
    digits(&mut s, "FF");
    s.set_mode(CalcMode::Basic);
    assert_eq!(s.value(), Ok(7.0));
    assert_eq!(s.memory, 7.0);
    s.set_mode(CalcMode::Programmer);
    assert_eq!(s.programmer.value, 255);
    assert_eq!(s.angle, AngleUnit::Radians);
    assert!(s.second_functions);
}
#[test]
fn bounded_expression_reports_overflow_and_clear_recovers() {
    let mut s = CalcState::new();
    for _ in 0..200 {
        s.open_parenthesis();
    }
    assert_eq!(s.active_error(), Some(CalcError::TooLong));
    s.clear();
    digits(&mut s, "2");
    assert_eq!(s.value(), Ok(2.0));
}
#[test]
fn programmer_radix_preserves_bits_and_rejects_invalid_digits() {
    let mut s = word("FF", 16);
    assert_eq!(s.value, 255);
    s.set_radix(10);
    assert_eq!(s.display(), "255");
    s.input_digit('F');
    assert_eq!(s.value, 255);
    s.set_radix(8);
    assert_eq!(s.display(), "377");
    s.input_digit('8');
    assert_eq!(s.value, 255);
}
#[test]
fn programmer_input_overflow_is_reported_without_wrapping() {
    let mut s = word("FFFFFFFFFFFFFFFF", 16);
    assert_eq!(s.value, u64::MAX);
    s.input_digit('F');
    assert_eq!(s.error, Some(CalcError::Overflow));
    s.input_digit('1');
    assert_eq!(s.value, 1);
    assert!(s.error.is_none());
    let s = word("9223372036854775808", 10);
    assert_eq!(s.error, Some(CalcError::Overflow));
}
#[test]
fn programmer_arithmetic_wraps_64_bits_and_integer_division_truncates() {
    let mut s = word("FFFFFFFFFFFFFFFF", 16);
    s.binary(B::Add);
    s.input_digit('1');
    s.calculate();
    assert_eq!(s.value, 0);
    let mut s = word("99", 10);
    s.binary(B::Divide);
    s.input_digit('1');
    s.input_digit('0');
    s.calculate();
    assert_eq!(s.value, 9);
    let mut s = word("99", 10);
    s.binary(B::Mod);
    s.input_digit('1');
    s.input_digit('0');
    s.calculate();
    assert_eq!(s.value, 9);
}
#[test]
fn programmer_bitwise_and_nor() {
    for (op, expected) in [
        (B::And, 0x30),
        (B::Or, 0xFC),
        (B::Xor, 0xCC),
        (B::Nor, !0xFCu64),
    ] {
        let mut s = word("F0", 16);
        s.binary(op);
        s.input_digit('3');
        s.input_digit('C');
        s.calculate();
        assert_eq!(s.value, expected);
    }
}
#[test]
fn programmer_shifts_are_logical_and_large_count_is_zero() {
    let mut s = word("8000000000000000", 16);
    s.unary(W::ShiftRight);
    assert_eq!(s.value, 0x4000000000000000);
    s.binary(B::ShiftLeft);
    s.input_digit('4');
    s.input_digit('0');
    s.calculate();
    assert_eq!(s.value, 0);
}
#[test]
fn programmer_rotate_and_flip_have_defined_lane_semantics() {
    let mut s = word("8000000000000001", 16);
    s.unary(W::RotateLeft);
    assert_eq!(s.value, 3);
    s.unary(W::RotateRight);
    assert_eq!(s.value, 0x8000000000000001);
    s.set_value(0x0123456789ABCDEF);
    s.unary(W::Flip8);
    assert_eq!(s.value, 0x23016745AB89EFCD);
    s.unary(W::Flip8);
    assert_eq!(s.value, 0x0123456789ABCDEF);
    s.unary(W::Flip16);
    assert_eq!(s.value, 0x45670123CDEF89AB);
    s.unary(W::Flip16);
    assert_eq!(s.value, 0x0123456789ABCDEF);
}
#[test]
fn programmer_negative_decimal_and_signed_division() {
    let mut s = word("9", 10);
    s.unary(W::Negate);
    assert_eq!(s.display(), "-9");
    s.binary(B::Divide);
    s.input_digit('2');
    s.calculate();
    assert_eq!(s.display(), "-4");
    s.set_radix(16);
    assert_eq!(s.display(), "FFFFFFFFFFFFFFFC");
}
#[test]
fn programmer_zero_division_and_parentheses() {
    let mut s = word("2", 16);
    s.binary(B::Divide);
    s.input_digit('0');
    s.calculate();
    assert_eq!(s.error, Some(CalcError::DivideByZero));
    s.clear();
    s.open_parenthesis();
    s.input_digit('3');
    s.binary(B::Add);
    s.input_digit('1');
    s.close_parenthesis();
    s.unary(W::Not);
    s.calculate();
    assert_eq!(s.value, !4u64);
}
#[test]
fn binary_view_can_toggle_high_and_low_bits_and_decode_characters() {
    let mut s = word("0", 16);
    s.toggle_bit(63);
    s.toggle_bit(0);
    assert_eq!(s.value, 0x8000000000000001);
    s.toggle_bit(64);
    assert_eq!(s.value, 0x8000000000000001);
    s.set_value(65);
    assert_eq!(s.character_display(), "ASCII  A");
    s.unicode = true;
    assert_eq!(s.character_display(), "U+0041  A");
    s.set_value(0xD800);
    assert_eq!(s.character_display(), "U+D800");
}
#[test]
fn secondary_functions_and_mode_actions_are_operational() {
    let mut s = CalcState::new();
    KeyAction::Mode(CalcMode::Scientific).apply(&mut s);
    KeyAction::Second.apply(&mut s);
    KeyAction::Angle.apply(&mut s);
    assert!(s.second_functions);
    assert_eq!(s.angle, AngleUnit::Radians);
    KeyAction::Mode(CalcMode::Programmer).apply(&mut s);
    KeyAction::Digits("FF").apply(&mut s);
    assert_eq!(s.programmer.value, 255);
}
#[test]
fn excessive_exponent_reports_overflow_and_parenthesized_percent_works() {
    let mut s = CalcState::new();
    digits(&mut s, "1");
    s.input_exponent();
    digits(&mut s, "999");
    s.calculate();
    assert_eq!(s.active_error(), Some(CalcError::Overflow));
    s.clear();
    s.open_parenthesis();
    digits(&mut s, "100");
    s.binary(B::Add);
    digits(&mut s, "15");
    s.percent();
    s.close_parenthesis();
    s.calculate();
    assert_eq!(s.value(), Ok(115.0));
}
#[test]
fn bit_edit_replaces_parenthesized_operand() {
    let mut s = word("0", 16);
    s.open_parenthesis();
    s.input_digit('1');
    s.close_parenthesis();
    s.toggle_bit(2);
    s.calculate();
    assert_eq!(s.value, 5);
}
