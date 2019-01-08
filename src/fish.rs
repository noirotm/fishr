extern crate serde;
#[macro_use]
extern crate serde_json;
extern crate rand;

use rand::{thread_rng, Rng, ThreadRng};
use serde_json::{to_value, Value};
use std::cmp;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::{stderr, BufReader, Bytes, Cursor};
use std::path::Path;

mod val;
pub use val::Val;

mod stack;
pub use stack::{Stack, StackOfStacks};

pub struct CodeBox {
    data: Vec<Vec<u8>>,
    height: usize,
    width: usize,
}

impl CodeBox {
    pub fn load<R: Read>(r: R) -> Result<CodeBox, Box<Error>> {
        let mut code_box = CodeBox {
            data: vec![],
            width: 0,
            height: 0,
        };
        for line in BufReader::new(r).lines() {
            code_box.push(line?.as_bytes().to_vec());
        }
        Ok(code_box)
    }

    pub fn load_from_file<P: AsRef<Path>>(filename: P) -> Result<CodeBox, Box<Error>> {
        let f = File::open(filename)?;
        Self::load(f)
    }

    pub fn load_from_string(s: &str) -> CodeBox {
        let b = Cursor::new(s);
        Self::load(b).expect("CodeBox::load_from_string failed")
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
        if x < self.width && y < self.height {
            let line = self.data.get(y).expect("line should not be None");
            Some(line.get(x).map_or(b' ', |c| *c))
        } else {
            None
        }
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
    memory_is_dirty: bool,
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
            output,
            rng: thread_rng(),
            state: ParserState::Normal,
            memory_is_dirty: false,
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

        let state = json!({
            "ip": vec![self.ip.chr, self.ip.line],

            "dir": match self.dir {
                Direction::Right => "right",
                Direction::Left => "left",
                Direction::Up => "up",
                Direction::Down => "down",
            },

            "next_instr": instruction as char,

            "stack": self.stack.top().values.iter().map(|val| match *val {
                Val::Byte(val) => to_value(val),
                Val::Int(val) => to_value(val),
                Val::Float(val) => to_value(val),
            }.unwrap_or(Value::Null)).collect::<Vec<_>>(),

            "register": self.stack.top().register.map_or(Value::Null, |val| match val {
                Val::Byte(val) => to_value(val),
                Val::Int(val) => to_value(val),
                Val::Float(val) => to_value(val),
            }.unwrap_or(Value::Null)),
        });

        writeln!(&mut stderr(), "{}", state.to_string()).expect("writeln! failed");
    }

    pub fn push_str(&mut self, s: &str) {
        for c in s.bytes() {
            self.stack.top().push(Val::Byte(c as u8));
        }
    }

    pub fn push_i64(&mut self, v: i64) {
        self.stack.top().push(Val::Int(v));
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
        // fetch from map only if memory is dirty
        if self.memory_is_dirty {
            // R/W codebox override (backed by a map)
            let pos = MemPos {
                x: self.ip.chr as i64,
                y: self.ip.line as i64,
            };
            if let Some(v) = self.memory.get(&pos) {
                return Some(v.to_u8());
            }
        }

        code.get(self.ip.chr, self.ip.line)
    }

    pub fn execute(
        &mut self,
        instruction: u8,
        code: &CodeBox,
    ) -> Result<RuntimeStatus, RuntimeError> {
        match self.state {
            ParserState::Normal => return self.execute_instruction(instruction, code),
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
        }
        Ok(RuntimeStatus::Continue)
    }

    #[inline]
    fn pop(&mut self) -> Result<Val, RuntimeError> {
        self.stack.top().pop().ok_or(RuntimeError::StackUnderflow)
    }

    fn execute_instruction(
        &mut self,
        instruction: u8,
        code: &CodeBox,
    ) -> Result<RuntimeStatus, RuntimeError> {
        match instruction {
            // Enter quote mode
            b'\'' => self.state = ParserState::SingleQuoted,
            b'"' => self.state = ParserState::DoubleQuoted,

            // # Movement and execution
            // absolute direction change
            b'>' => self.dir = Direction::Right,
            b'<' => self.dir = Direction::Left,
            b'^' => self.dir = Direction::Up,
            b'v' => self.dir = Direction::Down,

            // mirrors
            b'/' | b'\\' | b'|' | b'_' | b'#' => self.mirror(instruction),

            // random direction
            b'x' => {
                static DIRECTIONS: [Direction; 4] = [
                    Direction::Left,
                    Direction::Right,
                    Direction::Up,
                    Direction::Down,
                ];

                if let Some(dir) = self.rng.choose(&DIRECTIONS) {
                    self.dir = dir.clone();
                }
            }

            // skip the following instruction
            b'!' => self.advance(code),

            // Conditional trampoline - pop one value off the stack.
            // The next instruction is only executed if the popped value is non-zero.
            b'?' => {
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
            b'.' => self.jump(code)?,

            // # Literals and operators
            // literal values
            b'0'...b'9' | b'a'...b'f' => {
                if let Ok(val) = u8::from_str_radix(format!("{}", instruction as char).as_str(), 16) {
                    self.stack.top().push(Val::Byte(val));
                }
            }

            // arithmetic operations
            b'+' => self.add()?,
            b'-' => self.sub()?,
            b'*' => self.mul()?,
            b',' => self.div()?,
            b'%' => self.rem()?,

            // comparison tests
            b'=' => self.equals()?,
            b')' => self.gt()?,
            b'(' => self.lt()?,

            // # Stack manipulation
            // Duplicate the top value on the stack
            b':' => self
                .stack
                .top()
                .dup()
                .or(Err(RuntimeError::StackUnderflow))?,
            // Remove the top value from the stack
            b'~' => self
                .stack
                .top()
                .drop()
                .or(Err(RuntimeError::StackUnderflow))?,
            // Swap the top two values on the stack
            b'$' => self
                .stack
                .top()
                .swap()
                .or(Err(RuntimeError::StackUnderflow))?,
            // Swap the top three values on the stack
            b'@' => self
                .stack
                .top()
                .swap2()
                .or(Err(RuntimeError::StackUnderflow))?,
            // Shift the entire stack to the right
            b'}' => self.stack.top().rshift(),
            // Shift the entire stack to the left
            b'{' => self.stack.top().lshift(),
            // Reverse the stack
            b'r' => self.stack.top().values.reverse(),
            // Push the length of the stack onto the stack
            b'l' => {
                let l = self.stack.top().values.len();
                self.stack.top().values.push(Val::Int(l as i64));
            }

            // # Stack of stacks
            // Pop x off the stack and create a new stack, moving x values.
            b'[' => {
                let v = self.pop()?;
                self.stack
                    .push_stack(v.to_i64() as usize)
                    .or(Err(RuntimeError::StackUnderflow))?;
            }
            // Remove the current stack, moving its values to the top of the underlying stack
            b']' => self.stack.pop_stack(),

            // # I/O
            // Output value as character
            b'o' => self.char_output()?,
            // Output value as number
            b'n' => self.num_output()?,
            // Input byte
            b'i' => self.input()?,

            // register operation
            b'&' => self
                .stack
                .top()
                .switch_register()
                .or(Err(RuntimeError::StackUnderflow))?,

            // # Memory operations
            // Push from memory
            b'g' => self.read_memory(code)?,
            // Pop to memory
            b'p' => self.write_memory(code)?,

            // end execution
            b';' => return Ok(RuntimeStatus::Stop),

            // nop
            b' ' => {}

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
        match instruction {
            b'/' => {
                self.dir = match self.dir {
                    Direction::Right => Direction::Up,
                    Direction::Left => Direction::Down,
                    Direction::Up => Direction::Right,
                    Direction::Down => Direction::Left,
                }
            }
            b'\\' => {
                self.dir = match self.dir {
                    Direction::Right => Direction::Down,
                    Direction::Left => Direction::Up,
                    Direction::Up => Direction::Left,
                    Direction::Down => Direction::Right,
                }
            }
            b'|' => {
                self.dir = match self.dir {
                    Direction::Right => Direction::Left,
                    Direction::Left => Direction::Right,
                    Direction::Up => Direction::Up,
                    Direction::Down => Direction::Down,
                }
            }
            b'_' => {
                self.dir = match self.dir {
                    Direction::Right => Direction::Right,
                    Direction::Left => Direction::Left,
                    Direction::Up => Direction::Down,
                    Direction::Down => Direction::Up,
                }
            }
            b'#' => {
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
        let y = self.pop()?.to_i64();
        let x = self.pop()?.to_i64();

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

    fn add(&mut self) -> Result<(), RuntimeError> {
        let x = self.pop()?;
        let y = self.pop()?;

        let res = y.checked_add(x).ok_or(RuntimeError::IntegerOverflow)?;
        self.stack.top().push(res);
        Ok(())
    }

    fn sub(&mut self) -> Result<(), RuntimeError> {
        let x = self.pop()?;
        let y = self.pop()?;

        let res = y.checked_sub(x).ok_or(RuntimeError::IntegerOverflow)?;
        self.stack.top().push(res);
        Ok(())
    }

    fn mul(&mut self) -> Result<(), RuntimeError> {
        let x = self.pop()?;
        let y = self.pop()?;

        let res = y.checked_mul(x).ok_or(RuntimeError::IntegerOverflow)?;
        self.stack.top().push(res);
        Ok(())
    }

    fn div(&mut self) -> Result<(), RuntimeError> {
        let x = self.pop()?;
        let y = self.pop()?;

        let res = y.to_f64() / x.to_f64();
        if res.is_infinite() {
            return Err(RuntimeError::DivideByZero);
        }

        self.stack.top().push(Val::Float(res));
        Ok(())
    }

    fn rem(&mut self) -> Result<(), RuntimeError> {
        let x = self.pop()?.to_i64();
        let y = self.pop()?.to_i64();

        if x == 0 {
            return Err(RuntimeError::DivideByZero);
        }

        let rem = y % x;
        let modulo = rem.checked_add(x).ok_or(RuntimeError::IntegerOverflow)? % x;

        self.stack.top().push(Val::Int(modulo));
        Ok(())
    }

    fn equals(&mut self) -> Result<(), RuntimeError> {
        let x = self.pop()?.to_i64();
        let y = self.pop()?.to_i64();

        let res = y == x;
        self.stack.top().push(Val::Byte(res as u8));
        Ok(())
    }

    fn gt(&mut self) -> Result<(), RuntimeError> {
        let x = self.pop()?.to_i64();
        let y = self.pop()?.to_i64();

        let res = y > x;
        self.stack.top().push(Val::Byte(res as u8));
        Ok(())
    }

    fn lt(&mut self) -> Result<(), RuntimeError> {
        let x = self.pop()?.to_i64();
        let y = self.pop()?.to_i64();

        let res = y < x;
        self.stack.top().push(Val::Byte(res as u8));
        Ok(())
    }

    fn char_output(&mut self) -> Result<(), RuntimeError> {
        let c = self.pop()?.to_u8() as char;
        write!(&mut self.output, "{}", c).or(Err(RuntimeError::IOError))
    }

    fn num_output(&mut self) -> Result<(), RuntimeError> {
        match self.pop()? {
            Val::Float(f) => write!(&mut self.output, "{}", f).or(Err(RuntimeError::IOError)),
            v => write!(&mut self.output, "{}", v.to_i64()).or(Err(RuntimeError::IOError)),
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

    fn get_memory(&self, code: &CodeBox, x: i64, y: i64) -> Val {
        // fetch from map only if memory is dirty
        if self.memory_is_dirty {
            if let Some(v) = self.memory.get(&MemPos { x, y }) {
                return *v;
            }
        }

        let b = code.get(x as usize, y as usize);
        Val::Byte(match b {
            Some(b' ') | None => 0,
            Some(b) => b,
        })
    }

    fn read_memory(&mut self, code: &CodeBox) -> Result<(), RuntimeError> {
        let y = self.pop()?.to_i64();
        let x = self.pop()?.to_i64();

        let val = self.get_memory(code, x, y);
        self.stack.top().push(val);
        Ok(())
    }

    fn write_memory(&mut self, code: &CodeBox) -> Result<(), RuntimeError> {
        let y = self.pop()?.to_i64();
        let x = self.pop()?.to_i64();
        let v = self.pop()?;

        let val = self.get_memory(code, x, y);

        // abort if we don't actually change memory
        if v != val {
            self.memory.insert(MemPos { x, y }, v);
            self.memory_is_dirty = true;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{empty, sink};

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

    #[test]
    fn push_str_works() {
        let mut interpreter = Interpreter::new(empty(), sink());
        interpreter.push_str("foo");
        interpreter.push_str(" ");
        interpreter.push_str("bar");

        assert_eq!(
            interpreter.stack.top().values,
            vec![
                Val::Byte(b'f'),
                Val::Byte(b'o'),
                Val::Byte(b'o'),
                Val::Byte(b' '),
                Val::Byte(b'b'),
                Val::Byte(b'a'),
                Val::Byte(b'r'),
            ]
        );
    }

    #[test]
    fn push_i64_works() {
        let mut interpreter = Interpreter::new(empty(), sink());
        interpreter.push_i64(5);
        interpreter.push_i64(25);
        interpreter.push_i64(-45);

        assert_eq!(
            interpreter.stack.top().values,
            vec![Val::Int(5), Val::Int(25), Val::Int(-45)]
        );
    }
}
