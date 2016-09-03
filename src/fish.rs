use std::cmp;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;

extern crate rand;
use rand::{Rng, ThreadRng, thread_rng};

mod stack;
pub use stack::{Val, StackOfStacks, ValStack};

pub struct CodeBox {
    data: Vec<Vec<u8>>,
    height: usize,
    width: usize,
}

impl CodeBox {
    pub fn load_from_file<P: AsRef<Path>>(filename: P) -> Result<CodeBox, Box<Error>> {
        let f = try!(File::open(filename));
        let mut code_box = CodeBox {
            data: vec![],
            width: 0,
            height: 0,
        };
        for line in BufReader::new(f).lines() {
            let line = try!(line);
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

#[derive(Clone,Eq,PartialEq,Debug)]
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

#[derive(Eq,PartialEq,Debug,)]
pub enum RuntimeError {
    InvalidInstruction,
    InvalidIpPosition,
    StackUnderflow,
    DivideByZero,
}

enum ParserState {
    Normal,
    SingleQuoted,
    DoubleQuoted,
}

pub struct Interpreter<I: Read, O: Write> {
    pub ip: InstructionPtr,
    pub dir: Direction,
    pub stack: StackOfStacks,

    input: I,
    output: O,
    rng: ThreadRng,
    state: ParserState,
}

impl<I: Read, O: Write> Interpreter<I, O> {
    pub fn new(input: I, output: O) -> Interpreter<I, O> {
        Interpreter {
            ip: InstructionPtr {
                chr: 0,
                line: 0,
            },
            dir: Direction::Right,
            stack: StackOfStacks::new(),
            input: input,
            output: output,
            rng: thread_rng(),
            state: ParserState::Normal,
        }
    }

    pub fn reset(&mut self) {
        self.ip = InstructionPtr {
            chr: 0,
            line: 0,
        };
        self.dir = Direction::Right;
        self.state = ParserState::Normal;
    }

    pub fn run(&mut self, code: &CodeBox) -> Result<(), RuntimeError> {
        self.reset();
        loop {
            let instruction = match self.fetch(code) {
                Some(ch) => ch,
                None => return Err(RuntimeError::InvalidIpPosition),
            };

            //println!("{:?}", self.stack.stacks[0].values);
            //println!("[{}, {}] => {}", self.ip.chr, self.ip.line, instruction as char);

            match self.execute(instruction, code) {
                Ok(RuntimeStatus::Continue) => {},
                Ok(RuntimeStatus::Stop) => return Ok(()),
                Err(err) => return Err(err),
            }

            self.advance(code);
        }
    }

    pub fn fetch(&self, code: &CodeBox) -> Option<u8> {
        // here be R/W codebox override (backed by a map)
        code.get(self.ip.chr, self.ip.line)
    }

    pub fn execute(&mut self, instruction: u8, code: &CodeBox) -> Result<RuntimeStatus, RuntimeError> {
        match self.state {
            ParserState::SingleQuoted => match instruction as char {
                // Exit quote mode
                '\'' => self.state = ParserState::Normal,
                _ => self.stack.push(Val::Byte(instruction)),
            },
            ParserState::DoubleQuoted => match instruction as char {
                // Exit quote mode
                '"' => self.state = ParserState::Normal,
                _ => self.stack.push(Val::Byte(instruction)),
            },
            ParserState::Normal => return self.execute_instruction(instruction, code),
        }
        Ok(RuntimeStatus::Continue)
    }

    fn execute_instruction(&mut self, instruction: u8, code: &CodeBox) -> Result<RuntimeStatus, RuntimeError> {
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
                static DIRECTIONS: [Direction; 4] = [Direction::Left, Direction::Right, Direction::Up, Direction::Down];
                if let Some(dir) = self.rng.choose(&DIRECTIONS) {
                    self.dir = dir.clone();
                }
            },

            // skip the following instruction
            '!' => self.advance(code),

            // Conditional trampoline - pop one value off the stack.
            // The next instruction is only executed if the popped value is non-zero.
            '?' => {
                match self.stack.pop() {
                    Some(v) => if v.to_i64() == 0 {
                        self.advance(code);
                    },
                    None => return Err(RuntimeError::StackUnderflow),
                };
            },

            // jump to (x,y)
            '.' => try!(self.jump(code)),

            // # Literals and operators
            // literal values
            v @ '0' ... '9' | v @ 'a' ... 'f' => {
                if let Ok(val) = u8::from_str_radix(format!("{}", v).as_str(), 16) {
                    self.stack.push(Val::Byte(val));
                }
            },

            // arithmetic operations
            '+' => try!(self.add()),
            '-' => try!(self.sub()),
            '*' => try!(self.mul()),
            ',' => try!(self.div()),
            '%' => try!(self.rem()),

            // comparison tests
            '=' => try!(self.equals()),
            ')' => try!(self.gt()),
            '(' => try!(self.lt()),

            // # Stack manipulation
            // Duplicate the top value on the stack
            

            // end execution
            ';' => return Ok(RuntimeStatus::Stop),

            // nop
            ' ' => {},

            _ => return Err(RuntimeError::InvalidInstruction),
        }
        Ok(RuntimeStatus::Continue)
    }

    fn advance(&mut self, code: &CodeBox) {
        match self.dir {
            Direction::Right => self.ip.chr = self.ip.chr.checked_add(1).unwrap_or(0),
            Direction::Left => self.ip.chr = self.ip.chr.checked_sub(1).unwrap_or(code.width-1),
            Direction::Up => self.ip.line = self.ip.line.checked_sub(1).unwrap_or(code.height-1),
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
            '/' => self.dir = match self.dir {
                Direction::Right => Direction::Up,
                Direction::Left => Direction::Down,
                Direction::Up => Direction::Right,
                Direction::Down => Direction::Left,
            },
            '\\' => self.dir = match self.dir {
                Direction::Right => Direction::Down,
                Direction::Left => Direction::Up,
                Direction::Up => Direction::Left,
                Direction::Down => Direction::Right,
            },
            '|' => self.dir = match self.dir {
                Direction::Right => Direction::Left,
                Direction::Left => Direction::Right,
                Direction::Up => Direction::Up,
                Direction::Down => Direction::Down,
            },
            '_' => self.dir = match self.dir {
                Direction::Right => Direction::Right,
                Direction::Left => Direction::Left,
                Direction::Up => Direction::Down,
                Direction::Down => Direction::Up,
            },
            '#' => self.dir = match self.dir {
                Direction::Right => Direction::Left,
                Direction::Left => Direction::Right,
                Direction::Up => Direction::Down,
                Direction::Down => Direction::Up,
            },
            _ => {},
        }
    }

    fn jump(&mut self, code: &CodeBox) -> Result<(), RuntimeError> {
        match (self.stack.pop(), self.stack.pop()) {
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
            _ => Err(RuntimeError::StackUnderflow)
        }
    }

    fn add(&mut self) -> Result<(), RuntimeError> {
        match (self.stack.pop(), self.stack.pop()) {
            (Some(x), Some(y)) => {
                let res = y.to_i64() + x.to_i64();
                self.stack.push(Val::Int(res));
                Ok(())
            }
            _ => Err(RuntimeError::StackUnderflow)
        }
    }

    fn sub(&mut self) -> Result<(), RuntimeError> {
        match (self.stack.pop(), self.stack.pop()) {
            (Some(x), Some(y)) => {
                let res = y.to_i64() - x.to_i64();
                self.stack.push(Val::Int(res));
                Ok(())
            }
            _ => Err(RuntimeError::StackUnderflow)
        }
    }

    fn mul(&mut self) -> Result<(), RuntimeError> {
        match (self.stack.pop(), self.stack.pop()) {
            (Some(x), Some(y)) => {
                let res = y.to_i64() * x.to_i64();
                self.stack.push(Val::Int(res));
                Ok(())
            }
            _ => Err(RuntimeError::StackUnderflow)
        }
    }

    fn div(&mut self) -> Result<(), RuntimeError> {
        match (self.stack.pop(), self.stack.pop()) {
            (Some(ref x), _) if x.to_i64() == 0 => Err(RuntimeError::DivideByZero),

            (Some(x), Some(y)) => {
                let res = y.to_f64() / x.to_f64();
                self.stack.push(Val::Float(res));
                Ok(())
            }

            _ => Err(RuntimeError::StackUnderflow)
        }
    }

    fn rem(&mut self) -> Result<(), RuntimeError> {
        match (self.stack.pop(), self.stack.pop()) {
            (Some(ref x), _) if x.to_i64() == 0 => Err(RuntimeError::DivideByZero),

            (Some(x), Some(y)) => {
                let res = y.to_i64() % x.to_i64();
                self.stack.push(Val::Int(res));
                Ok(())
            }

            _ => Err(RuntimeError::StackUnderflow)
        }
    }

    fn equals(&mut self) -> Result<(), RuntimeError> {
        match (self.stack.pop(), self.stack.pop()) {
            (Some(x), Some(y)) => {
                let res = y.to_i64() == x.to_i64();
                self.stack.push(Val::Byte(match res {
                    true => 1,
                    false => 0,
                }));
                Ok(())
            }

            _ => Err(RuntimeError::StackUnderflow)
        }
    }

    fn gt(&mut self) -> Result<(), RuntimeError> {
        match (self.stack.pop(), self.stack.pop()) {
            (Some(x), Some(y)) => {
                let res = y.to_i64() > x.to_i64();
                self.stack.push(Val::Byte(match res {
                    true => 1,
                    false => 0,
                }));
                Ok(())
            }

            _ => Err(RuntimeError::StackUnderflow)
        }
    }

    fn lt(&mut self) -> Result<(), RuntimeError> {
        match (self.stack.pop(), self.stack.pop()) {
            (Some(x), Some(y)) => {
                let res = y.to_i64() < x.to_i64();
                self.stack.push(Val::Byte(match res {
                    true => 1,
                    false => 0,
                }));
                Ok(())
            }

            _ => Err(RuntimeError::StackUnderflow)
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
