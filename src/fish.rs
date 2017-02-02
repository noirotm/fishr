use std::cmp;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, Bytes};
use std::path::Path;

extern crate serde;
extern crate serde_json;

use serde_json::{Map, to_value};
use serde_json::Value;

extern crate rand;
use rand::{Rng, ThreadRng, thread_rng};

mod val;
pub use val::Val;

mod stack;
pub use stack::{StackOfStacks, Stack};

macro_rules! println_stderr(
    ($($arg:tt)*) => { {
        let r = writeln!(&mut ::std::io::stderr(), $($arg)*);
        r.expect("failed printing to stderr");
    } }
);

pub struct CodeBox {
    data: Vec<Vec<u8>>,
    height: usize,
    width: usize,
}

impl CodeBox {
    pub fn load_from_file<P: AsRef<Path>>(filename: P) -> Result<CodeBox, Box<Error>> {
        let f = File::open(filename)?;
        let mut code_box = CodeBox {
            data: vec![],
            width: 0,
            height: 0,
        };
        for line in BufReader::new(f).lines() {
            let line = line?;
            code_box.push(line.as_bytes().to_vec());
        }
        Ok(code_box)
    }

    pub fn load_from_string(s: &str) -> CodeBox {
        let mut code_box = CodeBox {
            data: vec![],
            width: 0,
            height: 0,
        };
        for line in s.lines() {
            code_box.push(line.as_bytes().to_vec());
        }
        code_box
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    fn push(&mut self, line: Vec<u8>) {
        self.height += 1;
        self.width = cmp::max(line.len(), self.width);
        self.data.push(line);
    }

    fn get(&self, x: usize, y: usize) -> Option<u8> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let line = self.data.get(y).unwrap();
        let ch = match line.get(x) {
            Some(c) => *c,
            None => ' ' as u8,
        };
        Some(ch)
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Direction {
    Right,
    Left,
    Up,
    Down,
}

pub struct InstructionPtr {
    pub chr: usize,
    pub line: usize,
}

pub enum RuntimeStatus {
    Continue,
    Stop,
}

#[derive(Eq, PartialEq, Debug)]
pub enum RuntimeError {
    InvalidInstruction,
    InvalidIpPosition,
    StackUnderflow,
    IntegerOverflow,
    DivideByZero,
    IOError,
}

enum ParserState {
    Normal,
    SingleQuoted,
    DoubleQuoted,
}

#[derive(Hash, Eq, PartialEq, Debug)]
pub struct MemPos {
    pub x: i64,
    pub y: i64,
}

pub struct Interpreter<R: Read, W: Write> {
    pub ip: InstructionPtr,
    pub dir: Direction,
    pub stack: StackOfStacks<Val>,
    pub memory: HashMap<MemPos, Val>,

    pub trace: bool,

    input: Bytes<R>,
    output: W,
    rng: ThreadRng,
    state: ParserState,
}

impl<R: Read, W: Write> Interpreter<R, W> {
    pub fn new(input: R, output: W) -> Interpreter<R, W> {
        Interpreter {
            ip: InstructionPtr { chr: 0, line: 0 },
            dir: Direction::Right,
            stack: StackOfStacks::new(),
            memory: HashMap::new(),
            trace: false,
            input: input.bytes(),
            output: output,
            rng: thread_rng(),
            state: ParserState::Normal,

        }
    }

    pub fn reset(&mut self) {
        self.ip = InstructionPtr { chr: 0, line: 0 };
        self.dir = Direction::Right;
        self.state = ParserState::Normal;
    }

    pub fn dump_state(&mut self, instruction: u8) {
        if instruction == b' ' {
            return;
        }

        let mut map = Map::new();

        // ip
        let ip = vec![self.ip.chr, self.ip.line];
        map.insert("ip", to_value(ip));
        map.insert("dir", to_value(match self.dir {
            Direction::Right => "right",
            Direction::Left => "left",
            Direction::Up => "up",
            Direction::Down => "down",
        }));

        // next instruction
        map.insert("next_instr", to_value(instruction as char));

        // stack
        let vals: Vec<_> = self.stack.top().values.iter().map(|val| match *val {
            Val::Byte(val) => to_value(val),
            Val::Int(val) => to_value(val),
            Val::Float(val) => to_value(val),
        }).collect();
        let reg = self.stack.top().register.map_or(Value::Null, |val| match val {
            Val::Byte(val) => to_value(val),
            Val::Int(val) => to_value(val),
            Val::Float(val) => to_value(val),
        });
        map.insert("stack", to_value(vals));

        // register
        map.insert("register", to_value(reg));

        let s = serde_json::to_string(&map).unwrap();
        println_stderr!("{}", s);
    }

    pub fn run(&mut self, code: &CodeBox) -> Result<(), RuntimeError> {
        self.reset();
        loop {
            let instruction = match self.fetch(code) {
                Some(ch) => ch,
                None => return Err(RuntimeError::InvalidIpPosition),
            };

            if self.trace {
                self.dump_state(instruction);
            }

            match self.execute(instruction, code) {
                Ok(RuntimeStatus::Continue) => {}
                Ok(RuntimeStatus::Stop) => return Ok(()),
                Err(err) => return Err(err),
            }

            self.advance(code);
        }
    }

    pub fn fetch(&self, code: &CodeBox) -> Option<u8> {
        // R/W codebox override (backed by a map)
        let pos = MemPos {
            x: self.ip.chr as i64,
            y: self.ip.line as i64,
        };
        match self.memory.get(&pos) {
            Some(v) => Some(v.to_u8()),
            None => code.get(self.ip.chr, self.ip.line),
        }
    }

    pub fn execute(&mut self,
                   instruction: u8,
                   code: &CodeBox)
                   -> Result<RuntimeStatus, RuntimeError> {
        match self.state {
            ParserState::SingleQuoted => {
                match instruction as char {
                    // Exit quote mode
                    '\'' => self.state = ParserState::Normal,
                    _ => self.stack.top().push(Val::Byte(instruction)),
                }
            }
            ParserState::DoubleQuoted => {
                match instruction as char {
                    // Exit quote mode
                    '"' => self.state = ParserState::Normal,
                    _ => self.stack.top().push(Val::Byte(instruction)),
                }
            }
            ParserState::Normal => return self.execute_instruction(instruction, code),
        }
        Ok(RuntimeStatus::Continue)
    }

    fn execute_instruction(&mut self,
                           instruction: u8,
                           code: &CodeBox)
                           -> Result<RuntimeStatus, RuntimeError> {
        match instruction as char {
            // Enter quote mode
            '\'' => self.state = ParserState::SingleQuoted,
            '"' => self.state = ParserState::DoubleQuoted,

            // # Movement and execution
            // absolute direction change
            '>' => self.dir = Direction::Right,
            '<' => self.dir = Direction::Left,
            '^' => self.dir = Direction::Up,
            'v' => self.dir = Direction::Down,

            // mirrors
            '/' | '\\' | '|' | '_' | '#' => self.mirror(instruction),

            // random direction
            'x' => {
                static DIRECTIONS: [Direction; 4] = [Direction::Left,
                                                     Direction::Right,
                                                     Direction::Up,
                                                     Direction::Down];

                if let Some(dir) = self.rng.choose(&DIRECTIONS) {
                    self.dir = dir.clone();
                }
            }

            // skip the following instruction
            '!' => self.advance(code),

            // Conditional trampoline - pop one value off the stack.
            // The next instruction is only executed if the popped value is non-zero.
            '?' => {
                match self.stack.top().pop() {
                    Some(v) => {
                        if v.to_i64() == 0 {
                            self.advance(code);
                        }
                    }
                    None => return Err(RuntimeError::StackUnderflow),
                };
            }

            // jump to (x,y)
            '.' => self.jump(code)?,

            // # Literals and operators
            // literal values
            v @ '0'...'9' | v @ 'a'...'f' => {
                if let Ok(val) = u8::from_str_radix(format!("{}", v).as_str(), 16) {
                    self.stack.top().push(Val::Byte(val));
                }
            }

            // arithmetic operations
            '+' => self.add()?,
            '-' => self.sub()?,
            '*' => self.mul()?,
            ',' => self.div()?,
            '%' => self.rem()?,

            // comparison tests
            '=' => self.equals()?,
            ')' => self.gt()?,
            '(' => self.lt()?,

            // # Stack manipulation
            // Duplicate the top value on the stack
            ':' => self.stack.top().dup().or(Err(RuntimeError::StackUnderflow))?,
            // Remove the top value from the stack
            '~' => self.stack.top().drop().or(Err(RuntimeError::StackUnderflow))?,
            // Swap the top two values on the stack
            '$' => self.stack.top().swap().or(Err(RuntimeError::StackUnderflow))?,
            // Swap the top three values on the stack
            '@' => self.stack.top().swap2().or(Err(RuntimeError::StackUnderflow))?,
            // Shift the entire stack to the right
            '}' => self.stack.top().rshift(),
            // Shift the entire stack to the left
            '{' => self.stack.top().lshift(),
            // Reverse the stack
            'r' => self.stack.top().values.reverse(),
            // Push the length of the stack onto the stack
            'l' => {
                let l = self.stack.top().values.len();
                self.stack.top().values.push(Val::Int(l as i64));
            }

            // # Stack of stacks
            // Pop x off the stack and create a new stack, moving x values.
            '[' => {
                match self.stack.top().pop() {
                    Some(v) => {
                        if let Err(_) = self.stack.push_stack(v.to_i64() as usize) {
                            return Err(RuntimeError::StackUnderflow);
                        }
                    }
                    None => return Err(RuntimeError::StackUnderflow),
                }
            }
            // Remove the current stack, moving its values to the top of the underlying stack
            ']' => self.stack.pop_stack(),

            // # I/O
            // Output value as character
            'o' => self.char_output()?,
            // Output value as number
            'n' => self.num_output()?,
            // Input byte
            'i' => self.input()?,

            // register operation
            '&' => self.stack.top().switch_register().or(Err(RuntimeError::StackUnderflow))?,

            // # Memory operations
            // Push from memory
            'g' => self.read_memory(code)?,
            // Pop to memory
            'p' => self.write_memory()?,

            // end execution
            ';' => return Ok(RuntimeStatus::Stop),

            // nop
            ' ' => {}

            _ => return Err(RuntimeError::InvalidInstruction),
        }
        Ok(RuntimeStatus::Continue)
    }

    fn advance(&mut self, code: &CodeBox) {
        match self.dir {
            Direction::Right => self.ip.chr = self.ip.chr.checked_add(1).unwrap_or(0),
            Direction::Left => self.ip.chr = self.ip.chr.checked_sub(1).unwrap_or(code.width - 1),
            Direction::Up => self.ip.line = self.ip.line.checked_sub(1).unwrap_or(code.height - 1),
            Direction::Down => self.ip.line = self.ip.line.checked_add(1).unwrap_or(0),
        }
        if self.ip.chr >= code.width {
            self.ip.chr = 0;
        }
        if self.ip.line >= code.height {
            self.ip.line = 0;
        }
    }

    fn mirror(&mut self, instruction: u8) {
        match instruction as char {
            '/' => {
                self.dir = match self.dir {
                    Direction::Right => Direction::Up,
                    Direction::Left => Direction::Down,
                    Direction::Up => Direction::Right,
                    Direction::Down => Direction::Left,
                }
            }
            '\\' => {
                self.dir = match self.dir {
                    Direction::Right => Direction::Down,
                    Direction::Left => Direction::Up,
                    Direction::Up => Direction::Left,
                    Direction::Down => Direction::Right,
                }
            }
            '|' => {
                self.dir = match self.dir {
                    Direction::Right => Direction::Left,
                    Direction::Left => Direction::Right,
                    Direction::Up => Direction::Up,
                    Direction::Down => Direction::Down,
                }
            }
            '_' => {
                self.dir = match self.dir {
                    Direction::Right => Direction::Right,
                    Direction::Left => Direction::Left,
                    Direction::Up => Direction::Down,
                    Direction::Down => Direction::Up,
                }
            }
            '#' => {
                self.dir = match self.dir {
                    Direction::Right => Direction::Left,
                    Direction::Left => Direction::Right,
                    Direction::Up => Direction::Down,
                    Direction::Down => Direction::Up,
                }
            }
            _ => {}
        }
    }

    fn jump(&mut self, code: &CodeBox) -> Result<(), RuntimeError> {
        match (self.stack.top().pop(), self.stack.top().pop()) {
            (Some(y_val), Some(x_val)) => {
                let y = y_val.to_i64();
                let x = x_val.to_i64();

                if x < 0 || y < 0 {
                    return Err(RuntimeError::InvalidIpPosition);
                }

                self.ip.chr = x as usize;
                self.ip.line = y as usize;

                if self.ip.chr >= code.width {
                    self.ip.chr = 0;
                }
                if self.ip.line >= code.height {
                    self.ip.line = 0;
                }

                Ok(())
            }
            _ => Err(RuntimeError::StackUnderflow),
        }
    }

    fn add(&mut self) -> Result<(), RuntimeError> {
        match (self.stack.top().pop(), self.stack.top().pop()) {
            (Some(x), Some(y)) => {
                let res = match y.checked_add(x) {
                    Some(v) => v,
                    None => return Err(RuntimeError::IntegerOverflow),
                };
                self.stack.top().push(res);
                Ok(())
            }
            _ => Err(RuntimeError::StackUnderflow),
        }
    }

    fn sub(&mut self) -> Result<(), RuntimeError> {
        match (self.stack.top().pop(), self.stack.top().pop()) {
            (Some(x), Some(y)) => {
                let res = match y.checked_sub(x) {
                    Some(v) => v,
                    None => return Err(RuntimeError::IntegerOverflow),
                };
                self.stack.top().push(res);
                Ok(())
            }
            _ => Err(RuntimeError::StackUnderflow),
        }
    }

    fn mul(&mut self) -> Result<(), RuntimeError> {
        match (self.stack.top().pop(), self.stack.top().pop()) {
            (Some(x), Some(y)) => {
                let res = match y.checked_mul(x) {
                    Some(v) => v,
                    None => return Err(RuntimeError::IntegerOverflow),
                };
                self.stack.top().push(res);
                Ok(())
            }
            _ => Err(RuntimeError::StackUnderflow),
        }
    }

    fn div(&mut self) -> Result<(), RuntimeError> {
        match (self.stack.top().pop(), self.stack.top().pop()) {
            (Some(ref x), _) if x.to_i64() == 0 => Err(RuntimeError::DivideByZero),

            (Some(x), Some(y)) => {
                let res = y.to_f64() / x.to_f64();
                self.stack.top().push(Val::Float(res));
                Ok(())
            }

            _ => Err(RuntimeError::StackUnderflow),
        }
    }

    fn rem(&mut self) -> Result<(), RuntimeError> {
        match (self.stack.top().pop(), self.stack.top().pop()) {
            (Some(ref x), _) if x.to_i64() == 0 => Err(RuntimeError::DivideByZero),

            (Some(x), Some(y)) => {
                let rem = y.to_i64() % x.to_i64();
                let modulo = match rem.checked_add(x.to_i64()) {
                    Some(s) => s % x.to_i64(),
                    _ => return Err(RuntimeError::IntegerOverflow),
                };
                self.stack.top().push(Val::Int(modulo));
                Ok(())
            }

            _ => Err(RuntimeError::StackUnderflow),
        }
    }

    fn equals(&mut self) -> Result<(), RuntimeError> {
        match (self.stack.top().pop(), self.stack.top().pop()) {
            (Some(x), Some(y)) => {
                let res = y.to_i64() == x.to_i64();
                self.stack.top().push(Val::Byte(match res {
                    true => 1,
                    false => 0,
                }));
                Ok(())
            }

            _ => Err(RuntimeError::StackUnderflow),
        }
    }

    fn gt(&mut self) -> Result<(), RuntimeError> {
        match (self.stack.top().pop(), self.stack.top().pop()) {
            (Some(x), Some(y)) => {
                let res = y.to_i64() > x.to_i64();
                self.stack.top().push(Val::Byte(match res {
                    true => 1,
                    false => 0,
                }));
                Ok(())
            }

            _ => Err(RuntimeError::StackUnderflow),
        }
    }

    fn lt(&mut self) -> Result<(), RuntimeError> {
        match (self.stack.top().pop(), self.stack.top().pop()) {
            (Some(x), Some(y)) => {
                let res = y.to_i64() < x.to_i64();
                self.stack.top().push(Val::Byte(match res {
                    true => 1,
                    false => 0,
                }));
                Ok(())
            }

            _ => Err(RuntimeError::StackUnderflow),
        }
    }

    fn char_output(&mut self) -> Result<(), RuntimeError> {
        match self.stack.top().pop() {
            Some(v) => {
                let c = v.to_u8() as char;
                write!(&mut self.output, "{}", c).or(Err(RuntimeError::IOError))
            }
            None => Err(RuntimeError::StackUnderflow),
        }
    }

    fn num_output(&mut self) -> Result<(), RuntimeError> {
        match self.stack.top().pop() {
            Some(Val::Float(f)) => write!(&mut self.output, "{}", f).or(Err(RuntimeError::IOError)),
            Some(v) => write!(&mut self.output, "{}", v.to_i64()).or(Err(RuntimeError::IOError)),
            None => Err(RuntimeError::StackUnderflow),
        }
    }

    fn input(&mut self) -> Result<(), RuntimeError> {
        match self.input.next() {
            Some(Ok(b)) => self.stack.top().push(Val::Byte(b)),
            Some(Err(_)) => return Err(RuntimeError::IOError),
            None => self.stack.top().push(Val::Int(-1)),
        }
        Ok(())
    }

    fn read_memory(&mut self, code: &CodeBox) -> Result<(), RuntimeError> {
        match (self.stack.top().pop(), self.stack.top().pop()) {
            (Some(y), Some(x)) => {
                let pos = MemPos {
                    x: x.to_i64(),
                    y: y.to_i64(),
                };
                let val = match self.memory.get(&pos) {
                    Some(&v) => v,
                    None => Val::Byte(match code.get(pos.x as usize, pos.y as usize) {
                        Some(b' ') | None => 0,
                        Some(b) => b,
                    }),
                };
                self.stack.top().push(val);
                Ok(())
            }
            _ => Err(RuntimeError::StackUnderflow),
        }
    }

    fn write_memory(&mut self) -> Result<(), RuntimeError> {
        match (self.stack.top().pop(), self.stack.top().pop(), self.stack.top().pop()) {
            (Some(y), Some(x), Some(v)) => {
                let pos = MemPos {
                    x: x.to_i64(),
                    y: y.to_i64(),
                };

                self.memory.insert(pos, v);
                Ok(())
            }
            _ => Err(RuntimeError::StackUnderflow),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codebox_with_one_line() {
        let cb = CodeBox::load_from_string("str");
        assert_eq!(cb.height, 1);
        assert_eq!(cb.width, 3);
        assert_eq!(cb.data.len(), 1);
    }

    #[test]
    fn codebox_with_one_column() {
        let cb = CodeBox::load_from_string("a\nb\nc\nd\ne");
        assert_eq!(cb.height, 5);
        assert_eq!(cb.width, 1);
        assert_eq!(cb.data.len(), 5);
    }

    #[test]
    fn codebox_data_is_ok() {
        let cb = CodeBox::load_from_string("str");
        assert_eq!(cb.data[0], vec![b's', b't', b'r']);
    }

    #[test]
    fn codebox_with_three_lines() {
        let cb = CodeBox::load_from_string("str\nmore\nlines");
        assert_eq!(cb.height, 3);
        assert_eq!(cb.width, 5);
    }

    #[test]
    fn empty_code_box() {
        let cb = CodeBox::load_from_string("");
        assert_eq!(cb.height, 0);
        assert_eq!(cb.width, 0);
        assert!(cb.data.is_empty());
    }
}
