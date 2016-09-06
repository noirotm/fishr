extern crate timebomb;
use timebomb::timeout_ms;

extern crate fish;

use std::io::{empty, sink};
use fish::*;

#[test]
fn run_exits() {
    timeout_ms(|| {
        let cb = CodeBox::load_from_string(";");
        let mut interpreter = Interpreter::new(empty(), sink());

        let result = interpreter.run(&cb);

        assert!(result.is_ok());
        assert_eq!(interpreter.ip.chr, 0);
        assert_eq!(interpreter.ip.line, 0);
        assert_eq!(interpreter.dir, Direction::Right);
    }, 1000);
}

#[test]
fn empty_code_does_not_run() {
    timeout_ms(|| {
        let cb = CodeBox::load_from_string("");
        let mut interpreter = Interpreter::new(empty(), sink());

        let result = interpreter.run(&cb);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), RuntimeError::InvalidIpPosition);
    }, 1000);
}

#[test]
fn invalid_code_does_not_run() {
    timeout_ms(|| {
        let cb = CodeBox::load_from_string("z"); // invalid opcode
        let mut interpreter = Interpreter::new(empty(), sink());

        let result = interpreter.run(&cb);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), RuntimeError::InvalidInstruction);
    }, 1000);
}

#[test]
fn skip_works() {
    timeout_ms(|| {
        let cb = CodeBox::load_from_string("!v;\n ;");
        let mut interpreter = Interpreter::new(empty(), sink());

        let result = interpreter.run(&cb);
        assert!(result.is_ok());
        assert_eq!(interpreter.dir, Direction::Right);
    }, 1000);
}

fn assert_expected_direction_eq(code: &'static str, expected_dir: Direction) {
    timeout_ms(move || {
        let cb = CodeBox::load_from_string(code);
        let mut interpreter = Interpreter::new(empty(), sink());

        let result = interpreter.run(&cb);

        assert!(result.is_ok());
        assert_eq!(interpreter.dir, expected_dir);
    }, 1000);
}

#[test]
fn move_right_works() {
    assert_expected_direction_eq(">;", Direction::Right);
}

#[test]
fn move_left_works() {
    assert_expected_direction_eq("<;", Direction::Left);
}

#[test]
fn move_down_works() {
    assert_expected_direction_eq("v\n;", Direction::Down);
}

#[test]
fn move_up_works() {
    assert_expected_direction_eq("^\n;", Direction::Up);
}

#[test]
fn mirror_right_to_down_works() {
    assert_expected_direction_eq(" \\\n ;", Direction::Down);
}

#[test]
fn mirror_right_to_up_works() {
    assert_expected_direction_eq(" /\n ;", Direction::Up);
}

#[test]
fn mirror_right_to_left_works() {
    assert_expected_direction_eq(" |;", Direction::Left);
}

#[test]
fn mirror_down_to_up_works() {
    assert_expected_direction_eq("v\n!\n;\n_", Direction::Up);
}

#[test]
fn mirror_multi_right_to_left_works() {
    assert_expected_direction_eq(" #;", Direction::Left);
}

#[test]
fn mirror_multi_down_to_up_works() {
    assert_expected_direction_eq("v\n!\n;\n#", Direction::Up);
}

#[test]
fn mirror_multi_up_to_down_works() {
    assert_expected_direction_eq("^\n#\n;\n!", Direction::Down);
}

#[test]
fn mirror_multi_left_to_right_works() {
    assert_expected_direction_eq("v\n<#;!", Direction::Right);
}

#[test]
fn conditional_trampoline_skips_next_with_zero() {
    timeout_ms(|| {
        let cb = CodeBox::load_from_string("0?1;");
        let mut interpreter = Interpreter::new(empty(), sink());

        let result = interpreter.run(&cb);

        assert!(result.is_ok());
        assert_eq!(interpreter.stack.top().values, vec![]);
    }, 1000);
}

#[test]
fn conditional_trampoline_executes_next_with_non_zero() {
    timeout_ms(|| {
        let cb = CodeBox::load_from_string("5?1;");
        let mut interpreter = Interpreter::new(empty(), sink());

        let result = interpreter.run(&cb);

        assert!(result.is_ok());
        assert_eq!(interpreter.stack.top().values, vec![Val::Byte(0x1)]);
    }, 1000);
}

#[test]
fn conditional_trampoline_with_empty_stack_fails() {
    timeout_ms(|| {
        let cb = CodeBox::load_from_string("?1;");
        let mut interpreter = Interpreter::new(empty(), sink());

        let result = interpreter.run(&cb);

        assert_eq!(result.unwrap_err(), RuntimeError::StackUnderflow);
    }, 1000);
}

#[test]
fn literal_works() {
    timeout_ms(|| {
        let cb = CodeBox::load_from_string("123abc;");
        let mut interpreter = Interpreter::new(empty(), sink());

        let result = interpreter.run(&cb);

        assert!(result.is_ok());
        assert_eq!(interpreter.stack.top().values, vec![
            Val::Byte(0x1), Val::Byte(0x2), Val::Byte(0x3), Val::Byte(0xa), Val::Byte(0xb), Val::Byte(0xc)
        ]);
    }, 1000);
}

#[test]
fn addition_works() {
    timeout_ms(|| {
        let cb = CodeBox::load_from_string("67+;");
        let mut interpreter = Interpreter::new(empty(), sink());

        let result = interpreter.run(&cb);

        assert!(result.is_ok());
        assert_eq!(interpreter.stack.top().values, vec![Val::Int(13)]);
    }, 1000);
}

#[test]
fn addition_with_empty_stack_fails() {
    timeout_ms(|| {
        let cb = CodeBox::load_from_string("+;");
        let mut interpreter = Interpreter::new(empty(), sink());

        let result = interpreter.run(&cb);

        assert_eq!(result.unwrap_err(), RuntimeError::StackUnderflow);
    }, 1000);
}

#[test]
fn addition_with_one_element_fails() {
    timeout_ms(|| {
        let cb = CodeBox::load_from_string("1+;");
        let mut interpreter = Interpreter::new(empty(), sink());

        let result = interpreter.run(&cb);

        assert_eq!(result.unwrap_err(), RuntimeError::StackUnderflow);
    }, 1000);
}

#[test]
fn substraction_works() {
    timeout_ms(|| {
        let cb = CodeBox::load_from_string("97- 79-;");
        let mut interpreter = Interpreter::new(empty(), sink());

        let result = interpreter.run(&cb);

        assert!(result.is_ok());
        assert_eq!(interpreter.stack.top().values, vec![Val::Int(2), Val::Int(-2)]);
    }, 1000);
}

#[test]
fn substraction_with_empty_stack_fails() {
    timeout_ms(|| {
        let cb = CodeBox::load_from_string("-;");
        let mut interpreter = Interpreter::new(empty(), sink());

        let result = interpreter.run(&cb);

        assert_eq!(result.unwrap_err(), RuntimeError::StackUnderflow);
    }, 1000);
}

#[test]
fn substraction_with_one_element_fails() {
    timeout_ms(|| {
        let cb = CodeBox::load_from_string("1-;");
        let mut interpreter = Interpreter::new(empty(), sink());

        let result = interpreter.run(&cb);

        assert_eq!(result.unwrap_err(), RuntimeError::StackUnderflow);
    }, 1000);
}

#[test]
fn multiplication_works() {
    timeout_ms(|| {
        let cb = CodeBox::load_from_string("67* 05*;");
        let mut interpreter = Interpreter::new(empty(), sink());

        let result = interpreter.run(&cb);

        assert!(result.is_ok());
        assert_eq!(interpreter.stack.top().values, vec![Val::Int(42), Val::Int(0)]);
    }, 1000);
}

#[test]
fn multiplication_with_empty_stack_fails() {
    timeout_ms(|| {
        let cb = CodeBox::load_from_string("*;");
        let mut interpreter = Interpreter::new(empty(), sink());

        let result = interpreter.run(&cb);

        assert_eq!(result.unwrap_err(), RuntimeError::StackUnderflow);
    }, 1000);
}

#[test]
fn multiplication_with_one_element_fails() {
    timeout_ms(|| {
        let cb = CodeBox::load_from_string("1*;");
        let mut interpreter = Interpreter::new(empty(), sink());

        let result = interpreter.run(&cb);

        assert_eq!(result.unwrap_err(), RuntimeError::StackUnderflow);
    }, 1000);
}

#[test]
fn division_works() {
    timeout_ms(|| {
        let cb = CodeBox::load_from_string("82, 94,;");
        let mut interpreter = Interpreter::new(empty(), sink());

        let result = interpreter.run(&cb);

        assert!(result.is_ok());
        assert_eq!(interpreter.stack.top().values, vec![Val::Float(4.0), Val::Float(2.25)]);
    }, 1000);
}

#[test]
fn division_with_empty_stack_fails() {
    timeout_ms(|| {
        let cb = CodeBox::load_from_string(",;");
        let mut interpreter = Interpreter::new(empty(), sink());

        let result = interpreter.run(&cb);

        assert_eq!(result.unwrap_err(), RuntimeError::StackUnderflow);
    }, 1000);
}

#[test]
fn division_with_one_element_fails() {
    timeout_ms(|| {
        let cb = CodeBox::load_from_string("1,;");
        let mut interpreter = Interpreter::new(empty(), sink());

        let result = interpreter.run(&cb);

        assert_eq!(result.unwrap_err(), RuntimeError::StackUnderflow);
    }, 1000);
}

#[test]
fn division_by_zero_fails() {
    timeout_ms(|| {
        let cb = CodeBox::load_from_string("50,;");
        let mut interpreter = Interpreter::new(empty(), sink());

        let result = interpreter.run(&cb);

        assert_eq!(result.unwrap_err(), RuntimeError::DivideByZero);
    }, 1000);
}

#[test]
fn modulo_works() {
    timeout_ms(|| {
        let cb = CodeBox::load_from_string("92% 84%;");
        let mut interpreter = Interpreter::new(empty(), sink());

        let result = interpreter.run(&cb);

        assert!(result.is_ok());
        assert_eq!(interpreter.stack.top().values, vec![Val::Int(1), Val::Int(0)]);
    }, 1000);
}

#[test]
fn modulo_with_empty_stack_fails() {
    timeout_ms(|| {
        let cb = CodeBox::load_from_string("%;");
        let mut interpreter = Interpreter::new(empty(), sink());

        let result = interpreter.run(&cb);

        assert_eq!(result.unwrap_err(), RuntimeError::StackUnderflow);
    }, 1000);
}

#[test]
fn modulo_with_one_element_fails() {
    timeout_ms(|| {
        let cb = CodeBox::load_from_string("1%;");
        let mut interpreter = Interpreter::new(empty(), sink());

        let result = interpreter.run(&cb);

        assert_eq!(result.unwrap_err(), RuntimeError::StackUnderflow);
    }, 1000);
}

#[test]
fn modulo_by_zero_fails() {
    timeout_ms(|| {
        let cb = CodeBox::load_from_string("50%;");
        let mut interpreter = Interpreter::new(empty(), sink());

        let result = interpreter.run(&cb);

        assert_eq!(result.unwrap_err(), RuntimeError::DivideByZero);
    }, 1000);
}

#[test]
fn single_quotes_work() {
    timeout_ms(|| {
        let cb = CodeBox::load_from_string("'abc\"';");
        let mut interpreter = Interpreter::new(empty(), sink());

        let result = interpreter.run(&cb);

        assert!(result.is_ok());
        assert_eq!(interpreter.stack.top().values, vec![
            Val::Byte(97), Val::Byte(98), Val::Byte(99), Val::Byte(34)
        ]);
    }, 1000);
}

#[test]
fn double_quotes_work() {
    timeout_ms(|| {
        let cb = CodeBox::load_from_string("\"abc'\";");
        let mut interpreter = Interpreter::new(empty(), sink());

        let result = interpreter.run(&cb);

        assert!(result.is_ok());
        assert_eq!(interpreter.stack.top().values, vec![
            Val::Byte(97), Val::Byte(98), Val::Byte(99), Val::Byte(39)
        ]);
    }, 1000);
}

#[test]
fn jump_works() {
    timeout_ms(|| {
        let cb = CodeBox::load_from_string("11.;\n  5;");
        let mut interpreter = Interpreter::new(empty(), sink());

        let result = interpreter.run(&cb);

        assert!(result.is_ok());
        assert_eq!(interpreter.stack.top().values, vec![Val::Byte(5)]);
    }, 1000);
}

#[test]
fn jump_with_empty_stack_fails() {
    timeout_ms(|| {
        let cb = CodeBox::load_from_string(".;");
        let mut interpreter = Interpreter::new(empty(), sink());

        let result = interpreter.run(&cb);

        assert_eq!(result.unwrap_err(), RuntimeError::StackUnderflow);
    }, 1000);
}

#[test]
fn jump_with_one_element_fails() {
    timeout_ms(|| {
        let cb = CodeBox::load_from_string("1.;");
        let mut interpreter = Interpreter::new(empty(), sink());

        let result = interpreter.run(&cb);

        assert_eq!(result.unwrap_err(), RuntimeError::StackUnderflow);
    }, 1000);
}

#[test]
fn jump_too_far_wraps_to_zero() {
    timeout_ms(|| {
        let cb = CodeBox::load_from_string("ff.;");
        let mut interpreter = Interpreter::new(empty(), sink());

        let result = interpreter.run(&cb);

        // after the jump that wraps to [0,0], the next ip position will be
        // [0,1] so we will execute jump again with only one value in the stack
        assert_eq!(result.unwrap_err(), RuntimeError::StackUnderflow);
    }, 1000);
}

#[test]
fn jump_to_negative_position_fails() {
    timeout_ms(|| {
        let cb = CodeBox::load_from_string("0f-0f-.;");
        let mut interpreter = Interpreter::new(empty(), sink());

        let result = interpreter.run(&cb);

        assert_eq!(result.unwrap_err(), RuntimeError::InvalidIpPosition);
    }, 1000);
}

#[test]
fn equal_works() {
    timeout_ms(|| {
        let cb = CodeBox::load_from_string("11= 01=;");
        let mut interpreter = Interpreter::new(empty(), sink());

        let result = interpreter.run(&cb);

        assert!(result.is_ok());
        assert_eq!(interpreter.stack.top().values, vec![Val::Byte(1), Val::Byte(0)]);
    }, 1000);
}

#[test]
fn equal_with_empty_stack_fails() {
    timeout_ms(|| {
        let cb = CodeBox::load_from_string("=;");
        let mut interpreter = Interpreter::new(empty(), sink());

        let result = interpreter.run(&cb);

        assert_eq!(result.unwrap_err(), RuntimeError::StackUnderflow);
    }, 1000);
}

#[test]
fn equal_with_one_element_fails() {
    timeout_ms(|| {
        let cb = CodeBox::load_from_string("1=;");
        let mut interpreter = Interpreter::new(empty(), sink());

        let result = interpreter.run(&cb);

        assert_eq!(result.unwrap_err(), RuntimeError::StackUnderflow);
    }, 1000);
}

#[test]
fn greater_than_works() {
    timeout_ms(|| {
        let cb = CodeBox::load_from_string("11) 01) 10);");
        let mut interpreter = Interpreter::new(empty(), sink());

        let result = interpreter.run(&cb);

        assert!(result.is_ok());
        assert_eq!(interpreter.stack.top().values, vec![Val::Byte(0), Val::Byte(0), Val::Byte(1)]);
    }, 1000);
}

#[test]
fn greater_than_with_empty_stack_fails() {
    timeout_ms(|| {
        let cb = CodeBox::load_from_string(");");
        let mut interpreter = Interpreter::new(empty(), sink());

        let result = interpreter.run(&cb);

        assert_eq!(result.unwrap_err(), RuntimeError::StackUnderflow);
    }, 1000);
}

#[test]
fn greater_than_with_one_element_fails() {
    timeout_ms(|| {
        let cb = CodeBox::load_from_string("1);");
        let mut interpreter = Interpreter::new(empty(), sink());

        let result = interpreter.run(&cb);

        assert_eq!(result.unwrap_err(), RuntimeError::StackUnderflow);
    }, 1000);
}

#[test]
fn less_than_works() {
    timeout_ms(|| {
        let cb = CodeBox::load_from_string("11( 01( 10(;");
        let mut interpreter = Interpreter::new(empty(), sink());

        let result = interpreter.run(&cb);

        assert!(result.is_ok());
        assert_eq!(interpreter.stack.top().values, vec![Val::Byte(0), Val::Byte(1), Val::Byte(0)]);
    }, 1000);
}

#[test]
fn less_than_with_empty_stack_fails() {
    timeout_ms(|| {
        let cb = CodeBox::load_from_string("(;");
        let mut interpreter = Interpreter::new(empty(), sink());

        let result = interpreter.run(&cb);

        assert_eq!(result.unwrap_err(), RuntimeError::StackUnderflow);
    }, 1000);
}

#[test]
fn less_than_with_one_element_fails() {
    timeout_ms(|| {
        let cb = CodeBox::load_from_string("1(;");
        let mut interpreter = Interpreter::new(empty(), sink());

        let result = interpreter.run(&cb);

        assert_eq!(result.unwrap_err(), RuntimeError::StackUnderflow);
    }, 1000);
}
