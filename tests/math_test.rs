extern crate fish;

use std::io::{empty, sink};
use fish::*;

#[test]
fn add_floats_yields_float() {
    let cb = CodeBox::load_from_string("92, 32, +;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert!(result.is_ok());
    assert_eq!(interpreter.stack.top().values, vec![Val::Float(6.0)]);
}

#[test]
fn add_float_and_int_yields_float() {
    let cb = CodeBox::load_from_string("92, 5 +;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert!(result.is_ok());
    assert_eq!(interpreter.stack.top().values, vec![Val::Float(9.5)]);
}

#[test]
fn add_int_and_float_yields_float() {
    let cb = CodeBox::load_from_string("5 92, +;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert!(result.is_ok());
    assert_eq!(interpreter.stack.top().values, vec![Val::Float(9.5)]);
}

#[test]
fn sub_floats_yields_float() {
    let cb = CodeBox::load_from_string("92, 32, -;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert!(result.is_ok());
    assert_eq!(interpreter.stack.top().values, vec![Val::Float(3.0)]);
}

#[test]
fn sub_float_and_int_yields_float() {
    let cb = CodeBox::load_from_string("92, 5 -;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert!(result.is_ok());
    assert_eq!(interpreter.stack.top().values, vec![Val::Float(-0.5)]);
}

#[test]
fn sub_int_and_float_yields_float() {
    let cb = CodeBox::load_from_string("5 92, -;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert!(result.is_ok());
    assert_eq!(interpreter.stack.top().values, vec![Val::Float(0.5)]);
}

#[test]
fn mul_floats_yields_float() {
    let cb = CodeBox::load_from_string("92, 32, *;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert!(result.is_ok());
    assert_eq!(interpreter.stack.top().values, vec![Val::Float(6.75)]);
}

#[test]
fn mul_float_and_int_yields_float() {
    let cb = CodeBox::load_from_string("92, 5 *;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert!(result.is_ok());
    assert_eq!(interpreter.stack.top().values, vec![Val::Float(22.5)]);
}

#[test]
fn mul_int_and_float_yields_float() {
    let cb = CodeBox::load_from_string("5 92, *;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert!(result.is_ok());
    assert_eq!(interpreter.stack.top().values, vec![Val::Float(22.5)]);
}

#[test]
fn mod_negative_value_works() {
    // python uses true modulo whereas Rust, just like C, uses remainder.
    // Therefore we have to check that our implementation conforms
    // with Python's behaviour, eg. -1 % 13 = 12
    let cb = CodeBox::load_from_string("01- d %;");
    let mut interpreter = Interpreter::new(empty(), sink());

    let result = interpreter.run(&cb);

    assert!(result.is_ok());
    assert_eq!(interpreter.stack.top().values, vec![Val::Int(12)]);
}
