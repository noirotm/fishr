extern crate fish;

use std::io::{empty, sink};
use fish::*;

#[test]
fn end_works() {
    let cb = CodeBox::load_from_string(";");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert!(result.is_ok());
    assert_eq!(interpreter.ip.chr, 0);
    assert_eq!(interpreter.ip.line, 0);
    assert_eq!(interpreter.dir, Direction::Right);
}

#[test]
fn empty_code_does_not_run() {
    let cb = CodeBox::load_from_string("");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert_eq!(result, Err(RuntimeError::InvalidIpPosition));
}

#[test]
fn invalid_code_does_not_run() {
    let cb = CodeBox::load_from_string("z"); // invalid opcode
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert_eq!(result, Err(RuntimeError::InvalidInstruction));
}

#[test]
fn skip_works() {
    let cb = CodeBox::load_from_string("!v;\n ;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert!(result.is_ok());
    assert_eq!(interpreter.dir, Direction::Right);
}

fn assert_expected_direction_eq(code: &'static str, expected_dir: Direction) {
    let cb = CodeBox::load_from_string(code);
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert!(result.is_ok());
    assert_eq!(interpreter.dir, expected_dir);
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
    let cb = CodeBox::load_from_string("0?1;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert!(result.is_ok());
    assert_eq!(interpreter.stack.top().values, vec![]);
}

#[test]
fn conditional_trampoline_executes_next_with_non_zero() {
    let cb = CodeBox::load_from_string("5?1;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert!(result.is_ok());
    assert_eq!(interpreter.stack.top().values, vec![Val::Byte(0x1)]);
}

#[test]
fn conditional_trampoline_with_empty_stack_fails() {
    let cb = CodeBox::load_from_string("?1;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert_eq!(result, Err(RuntimeError::StackUnderflow));
}

#[test]
fn literal_works() {
    let cb = CodeBox::load_from_string("123abc;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert!(result.is_ok());
    assert_eq!(interpreter.stack.top().values,
               vec![Val::Byte(0x1),
                    Val::Byte(0x2),
                    Val::Byte(0x3),
                    Val::Byte(0xa),
                    Val::Byte(0xb),
                    Val::Byte(0xc)]);
}

#[test]
fn addition_works() {
    let cb = CodeBox::load_from_string("67+;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert!(result.is_ok());
    assert_eq!(interpreter.stack.top().values, vec![Val::Int(13)]);
}

#[test]
fn addition_with_empty_stack_fails() {
    let cb = CodeBox::load_from_string("+;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert_eq!(result, Err(RuntimeError::StackUnderflow));
}

#[test]
fn addition_with_one_element_fails() {
    let cb = CodeBox::load_from_string("1+;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert_eq!(result, Err(RuntimeError::StackUnderflow));
}

#[test]
fn substraction_works() {
    let cb = CodeBox::load_from_string("97- 79-;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert!(result.is_ok());
    assert_eq!(interpreter.stack.top().values,
               vec![Val::Int(2), Val::Int(-2)]);
}

#[test]
fn substraction_with_empty_stack_fails() {
    let cb = CodeBox::load_from_string("-;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert_eq!(result, Err(RuntimeError::StackUnderflow));
}

#[test]
fn substraction_with_one_element_fails() {
    let cb = CodeBox::load_from_string("1-;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert_eq!(result, Err(RuntimeError::StackUnderflow));
}

#[test]
fn multiplication_works() {
    let cb = CodeBox::load_from_string("67* 05*;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert!(result.is_ok());
    assert_eq!(interpreter.stack.top().values,
               vec![Val::Int(42), Val::Int(0)]);
}

#[test]
fn multiplication_with_empty_stack_fails() {
    let cb = CodeBox::load_from_string("*;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert_eq!(result, Err(RuntimeError::StackUnderflow));
}

#[test]
fn multiplication_with_one_element_fails() {
    let cb = CodeBox::load_from_string("1*;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert_eq!(result, Err(RuntimeError::StackUnderflow));
}

#[test]
fn division_works() {
    let cb = CodeBox::load_from_string("82, 94,;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert!(result.is_ok());
    assert_eq!(interpreter.stack.top().values,
               vec![Val::Float(4.0), Val::Float(2.25)]);
}

#[test]
fn division_with_empty_stack_fails() {
    let cb = CodeBox::load_from_string(",;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert_eq!(result, Err(RuntimeError::StackUnderflow));
}

#[test]
fn division_with_one_element_fails() {
    let cb = CodeBox::load_from_string("1,;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert_eq!(result, Err(RuntimeError::StackUnderflow));
}

#[test]
fn division_by_zero_fails() {
    let cb = CodeBox::load_from_string("50,;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert_eq!(result, Err(RuntimeError::DivideByZero));
}

#[test]
fn modulo_works() {
    let cb = CodeBox::load_from_string("92% 84%;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert!(result.is_ok());
    assert_eq!(interpreter.stack.top().values,
               vec![Val::Int(1), Val::Int(0)]);
}

#[test]
fn modulo_with_empty_stack_fails() {
    let cb = CodeBox::load_from_string("%;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert_eq!(result, Err(RuntimeError::StackUnderflow));
}

#[test]
fn modulo_with_one_element_fails() {
    let cb = CodeBox::load_from_string("1%;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert_eq!(result, Err(RuntimeError::StackUnderflow));
}

#[test]
fn modulo_by_zero_fails() {
    let cb = CodeBox::load_from_string("50%;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert_eq!(result, Err(RuntimeError::DivideByZero));
}

#[test]
fn single_quotes_work() {
    let cb = CodeBox::load_from_string("'abc\"';");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert!(result.is_ok());
    assert_eq!(interpreter.stack.top().values,
               vec![Val::Byte(97), Val::Byte(98), Val::Byte(99), Val::Byte(34)]);
}

#[test]
fn double_quotes_work() {
    let cb = CodeBox::load_from_string("\"abc'\";");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert!(result.is_ok());
    assert_eq!(interpreter.stack.top().values,
               vec![Val::Byte(97), Val::Byte(98), Val::Byte(99), Val::Byte(39)]);
}

#[test]
fn jump_works() {
    let cb = CodeBox::load_from_string("11.;\n  5;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert!(result.is_ok());
    assert_eq!(interpreter.stack.top().values, vec![Val::Byte(5)]);
}

#[test]
fn jump_with_empty_stack_fails() {
    let cb = CodeBox::load_from_string(".;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert_eq!(result, Err(RuntimeError::StackUnderflow));
}

#[test]
fn jump_with_one_element_fails() {
    let cb = CodeBox::load_from_string("1.;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert_eq!(result, Err(RuntimeError::StackUnderflow));
}

#[test]
fn jump_too_far_wraps_to_zero() {
    let cb = CodeBox::load_from_string("ff.;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    // after the jump that wraps to [0,0], the next ip position will be
    // [0,1] so we will execute jump again with only one value in the stack
    assert_eq!(result, Err(RuntimeError::StackUnderflow));
}

#[test]
fn jump_to_negative_position_fails() {
    let cb = CodeBox::load_from_string("0f-0f-.;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert_eq!(result, Err(RuntimeError::InvalidIpPosition));
}

#[test]
fn equal_works() {
    let cb = CodeBox::load_from_string("11= 01=;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert!(result.is_ok());
    assert_eq!(interpreter.stack.top().values,
               vec![Val::Byte(1), Val::Byte(0)]);
}

#[test]
fn equal_with_empty_stack_fails() {
    let cb = CodeBox::load_from_string("=;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert_eq!(result, Err(RuntimeError::StackUnderflow));
}

#[test]
fn equal_with_one_element_fails() {
    let cb = CodeBox::load_from_string("1=;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert_eq!(result, Err(RuntimeError::StackUnderflow));
}

#[test]
fn greater_than_works() {
    let cb = CodeBox::load_from_string("11) 01) 10);");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert!(result.is_ok());
    assert_eq!(interpreter.stack.top().values,
               vec![Val::Byte(0), Val::Byte(0), Val::Byte(1)]);
}

#[test]
fn greater_than_with_empty_stack_fails() {
    let cb = CodeBox::load_from_string(");");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert_eq!(result, Err(RuntimeError::StackUnderflow));
}

#[test]
fn greater_than_with_one_element_fails() {
    let cb = CodeBox::load_from_string("1);");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert_eq!(result, Err(RuntimeError::StackUnderflow));
}

#[test]
fn less_than_works() {
    let cb = CodeBox::load_from_string("11( 01( 10(;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert!(result.is_ok());
    assert_eq!(interpreter.stack.top().values,
               vec![Val::Byte(0), Val::Byte(1), Val::Byte(0)]);
}

#[test]
fn less_than_with_empty_stack_fails() {
    let cb = CodeBox::load_from_string("(;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert_eq!(result, Err(RuntimeError::StackUnderflow));
}

#[test]
fn less_than_with_one_element_fails() {
    let cb = CodeBox::load_from_string("1(;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert_eq!(result, Err(RuntimeError::StackUnderflow));
}

#[test]
fn dup_works() {
    let cb = CodeBox::load_from_string("123:;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert!(result.is_ok());
    assert_eq!(interpreter.stack.top().values,
               vec![Val::Byte(1), Val::Byte(2), Val::Byte(3), Val::Byte(3)]);
}

#[test]
fn dup_with_empty_stack_fails() {
    let cb = CodeBox::load_from_string(":;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert_eq!(result, Err(RuntimeError::StackUnderflow));
}

#[test]
fn drop_works() {
    let cb = CodeBox::load_from_string("123~;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert!(result.is_ok());
    assert_eq!(interpreter.stack.top().values,
               vec![Val::Byte(1), Val::Byte(2)]);
}

#[test]
fn drop_with_empty_stack_fails() {
    let cb = CodeBox::load_from_string("~;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert_eq!(result, Err(RuntimeError::StackUnderflow));
}

#[test]
fn swap_works() {
    let cb = CodeBox::load_from_string("123$;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert!(result.is_ok());
    assert_eq!(interpreter.stack.top().values,
               vec![Val::Byte(1), Val::Byte(3), Val::Byte(2)]);
}

#[test]
fn swap_with_empty_stack_fails() {
    let cb = CodeBox::load_from_string("$;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert_eq!(result, Err(RuntimeError::StackUnderflow));
}

#[test]
fn swap_with_one_element_fails() {
    let cb = CodeBox::load_from_string("1$;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert_eq!(result, Err(RuntimeError::StackUnderflow));
}

#[test]
fn swap2_works() {
    let cb = CodeBox::load_from_string("1234@;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert!(result.is_ok());
    assert_eq!(interpreter.stack.top().values,
               vec![Val::Byte(1), Val::Byte(4), Val::Byte(2), Val::Byte(3)]);
}

#[test]
fn swap2_with_empty_stack_fails() {
    let cb = CodeBox::load_from_string("@;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert_eq!(result, Err(RuntimeError::StackUnderflow));
}

#[test]
fn swap2_with_one_element_fails() {
    let cb = CodeBox::load_from_string("1@;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert_eq!(result, Err(RuntimeError::StackUnderflow));
}

#[test]
fn swap2_with_two_elements_fails() {
    let cb = CodeBox::load_from_string("12@;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert_eq!(result, Err(RuntimeError::StackUnderflow));
}

#[test]
fn rshift_works() {
    let cb = CodeBox::load_from_string("1234};");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert!(result.is_ok());
    assert_eq!(interpreter.stack.top().values,
               vec![Val::Byte(4), Val::Byte(1), Val::Byte(2), Val::Byte(3)]);
}

#[test]
fn lshift_works() {
    let cb = CodeBox::load_from_string("1234{;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert!(result.is_ok());
    assert_eq!(interpreter.stack.top().values,
               vec![Val::Byte(2), Val::Byte(3), Val::Byte(4), Val::Byte(1)]);
}

#[test]
fn reverse_works() {
    let cb = CodeBox::load_from_string("1234r;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert!(result.is_ok());
    assert_eq!(interpreter.stack.top().values,
               vec![Val::Byte(4), Val::Byte(3), Val::Byte(2), Val::Byte(1)]);
}

#[test]
fn len_works() {
    let cb = CodeBox::load_from_string("1234l;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert!(result.is_ok());
    assert_eq!(interpreter.stack.top().values,
               vec![Val::Byte(1), Val::Byte(2), Val::Byte(3), Val::Byte(4), Val::Int(4)]);
}

#[test]
fn len_with_empty_stack_works() {
    let cb = CodeBox::load_from_string("l;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert!(result.is_ok());
    assert_eq!(interpreter.stack.top().values, vec![Val::Int(0)]);
}

#[test]
fn new_stack_works() {
    let cb = CodeBox::load_from_string("1234 2[;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert!(result.is_ok());
    assert_eq!(interpreter.stack.top().values,
               vec![Val::Byte(3), Val::Byte(4)]);
}

#[test]
fn new_empty_stack_works() {
    let cb = CodeBox::load_from_string("1234 0[;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert!(result.is_ok());
    assert_eq!(interpreter.stack.top().values, vec![]);
}

#[test]
fn new_stack_with_too_many_elements_fails() {
    let cb = CodeBox::load_from_string("123 5[;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert_eq!(result, Err(RuntimeError::StackUnderflow));
}

#[test]
fn new_stack_with_negative_elements_fails() {
    let cb = CodeBox::load_from_string("123 15-[;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert_eq!(result, Err(RuntimeError::StackUnderflow));
}

#[test]
fn remove_stack_works() {
    let cb = CodeBox::load_from_string("1234 0[ 5678];");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert!(result.is_ok());
    assert_eq!(interpreter.stack.top().values,
               vec![Val::Byte(1),
                    Val::Byte(2),
                    Val::Byte(3),
                    Val::Byte(4),
                    Val::Byte(5),
                    Val::Byte(6),
                    Val::Byte(7),
                    Val::Byte(8)]);
}

#[test]
fn remove_last_stack_empties_it() {
    let cb = CodeBox::load_from_string("1234];");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert!(result.is_ok());
    assert_eq!(interpreter.stack.top().values, vec![]);
}

#[test]
fn char_output_works() {
    let mut out = Vec::new();
    let cb = CodeBox::load_from_string("\"1\"o;");
    {
        let mut interpreter = Interpreter::new(empty(), &mut out);

        let result = interpreter.run(&cb);

        assert!(result.is_ok());
        assert_eq!(interpreter.stack.top().values, vec![]);
    }
    assert_eq!(out, b"1");
}

#[test]
fn char_output_with_empty_stack_fails() {
    let mut out = Vec::new();
    let cb = CodeBox::load_from_string("o;");

    let mut interpreter = Interpreter::new(empty(), &mut out);

    let result = interpreter.run(&cb);

    assert_eq!(result, Err(RuntimeError::StackUnderflow));
}

#[test]
fn num_output_int_works() {
    let mut out = Vec::new();
    let cb = CodeBox::load_from_string("67*n;");
    {
        let mut interpreter = Interpreter::new(empty(), &mut out);

        let result = interpreter.run(&cb);

        assert!(result.is_ok());
        assert_eq!(interpreter.stack.top().values, vec![]);
    }
    assert_eq!(out, b"42");
}

#[test]
fn num_output_float_works() {
    let mut out = Vec::new();
    let cb = CodeBox::load_from_string("92,n;");
    {
        let mut interpreter = Interpreter::new(empty(), &mut out);

        let result = interpreter.run(&cb);

        assert!(result.is_ok());
        assert_eq!(interpreter.stack.top().values, vec![]);
    }
    assert_eq!(out, b"4.5");
}

#[test]
fn num_output_with_empty_stack_fails() {
    let mut out = Vec::new();
    let cb = CodeBox::load_from_string("n;");

    let mut interpreter = Interpreter::new(empty(), &mut out);

    let result = interpreter.run(&cb);

    assert_eq!(result, Err(RuntimeError::StackUnderflow));
}

#[test]
fn input_works() {
    let input: &[u8] = b"123";
    let cb = CodeBox::load_from_string("iiii;");

    let mut interpreter = Interpreter::new(input, sink());

    let result = interpreter.run(&cb);

    assert!(result.is_ok());
    assert_eq!(interpreter.stack.top().values,
               vec![Val::Byte(49), Val::Byte(50), Val::Byte(51), Val::Int(-1)]);
}

#[test]
fn switch_register_from_empty_works() {
    let cb = CodeBox::load_from_string("1234&;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert!(result.is_ok());
    assert_eq!(interpreter.stack.top().values, vec![
        Val::Byte(1), Val::Byte(2), Val::Byte(3)
    ]);
    assert_eq!(interpreter.stack.top().register, Some(Val::Byte(4)));
}

#[test]
fn switch_register_from_full_works() {
    let cb = CodeBox::load_from_string("1234& &;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert!(result.is_ok());
    assert_eq!(interpreter.stack.top().values, vec![
        Val::Byte(1), Val::Byte(2), Val::Byte(3), Val::Byte(4)
    ]);
    assert_eq!(interpreter.stack.top().register, None);
}

#[test]
fn switch_register_with_empty_stack_fails() {
    let mut out = Vec::new();
    let cb = CodeBox::load_from_string("&;");

    let mut interpreter = Interpreter::new(empty(), &mut out);

    let result = interpreter.run(&cb);

    assert_eq!(result, Err(RuntimeError::StackUnderflow));
}

#[test]
fn read_memory_works() {
    let cb = CodeBox::load_from_string("50g; 8");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert!(result.is_ok());
    assert_eq!(interpreter.stack.top().values, vec![Val::Byte(56)]);
}

#[test]
fn read_memory_outside_codebox_pushes_zero() {
    let cb = CodeBox::load_from_string("99g 09-09-g;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert!(result.is_ok());
    assert_eq!(interpreter.stack.top().values, vec![Val::Byte(0), Val::Byte(0)]);
}

#[test]
fn read_memory_with_space_pushes_zero() {
    let cb = CodeBox::load_from_string(" 00g;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert!(result.is_ok());
    assert_eq!(interpreter.stack.top().values, vec![Val::Byte(0)]);
}

#[test]
fn read_memory_with_empty_stack_fails() {
    let mut out = Vec::new();
    let cb = CodeBox::load_from_string("g;");

    let mut interpreter = Interpreter::new(empty(), &mut out);

    let result = interpreter.run(&cb);

    assert_eq!(result, Err(RuntimeError::StackUnderflow));
}

#[test]
fn read_memory_with_one_element_fails() {
    let mut out = Vec::new();
    let cb = CodeBox::load_from_string("0g;");

    let mut interpreter = Interpreter::new(empty(), &mut out);

    let result = interpreter.run(&cb);

    assert_eq!(result, Err(RuntimeError::StackUnderflow));
}

#[test]
fn write_memory_works() {
    let cb = CodeBox::load_from_string("599p 99g;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert!(result.is_ok());
    assert_eq!(interpreter.stack.top().values, vec![Val::Byte(5)]);
    assert_eq!(interpreter.memory[&MemPos{x: 9, y: 9}], Val::Byte(5));
}

#[test]
fn write_memory_with_empty_stack_fails() {
    let mut out = Vec::new();
    let cb = CodeBox::load_from_string("p;");

    let mut interpreter = Interpreter::new(empty(), &mut out);

    let result = interpreter.run(&cb);

    assert_eq!(result, Err(RuntimeError::StackUnderflow));
}
