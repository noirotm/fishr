#[derive(PartialEq, Debug)]
pub enum Val {
    Byte(u8),
    Int(i64),
    Float(f64),
}

impl Val {
    pub fn to_i64(&self) -> i64 {
        match *self {
            Val::Byte(val) => val as i64,
            Val::Int(val) => val,
            Val::Float(val) => val as i64,
        }
    }

    pub fn to_u8(&self) -> u8 {
        match *self {
            Val::Byte(val) => val,
            Val::Int(val) => val as u8,
            Val::Float(val) => val as u8,
        }
    }

    pub fn to_f64(&self) -> f64 {
        match *self {
            Val::Byte(val) => val as f64,
            Val::Int(val) => val as f64,
            Val::Float(val) => val,
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum Error {
    StackTooSmall,
}

pub struct ValStack {
    pub values: Vec<Val>,
    pub register: Option<Val>,
}

impl ValStack {
    pub fn new() -> ValStack {
        ValStack {
            values: Vec::new(),
            register: None,
        }
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn has_at_least(&self, n: usize) -> bool {
        self.values.len() >= n
    }

    pub fn push(&mut self, val: Val) {
        self.values.push(val);
    }

    pub fn pop(&mut self) -> Option<Val> {
        self.values.pop()
    }

    pub fn set_register_value(&mut self, val: Val) {
        self.register = Some(val);
    }

    pub fn clear_register(&mut self) {
        self.register = None;
    }
}

pub struct StackOfStacks {
    pub stacks: Vec<ValStack>
}

impl StackOfStacks {
    pub fn new() -> StackOfStacks {
        StackOfStacks{
            stacks: vec![ValStack::new()], // there is always at least one stack
        }
    }

    pub fn push_stack(&mut self, moved_items: usize) -> Result<(), Error> {
        let vals: Vec<_> = {
            let stack = self.top();
            match stack.len().checked_sub(moved_items) {
                Some(v) => stack.values.split_off(v),
                None => return Err(Error::StackTooSmall),
            }
        };

        self.stacks.push(ValStack {
            values: vals,
            register: None,
        });

        Ok(())
    }

    pub fn pop_stack(&mut self) {
        if self.stacks.len() > 1 {
            let v = self.stacks.pop().unwrap().values;
            self.top().values.extend(v);
        } else {
            let s = self.top();
            s.values.clear();
            s.register = None;
        }
    }

    pub fn switch_register(&mut self) -> Result<(), Error> {
        let stack = self.top();

        match stack.register.take() {
            None => match stack.pop() {
                Some(val) => stack.register = Some(val),
                None => return Err(Error::StackTooSmall),
            },
            Some(val) => stack.push(val),
        }

        Ok(())
    }

    pub fn top(&mut self) -> &mut ValStack {
        debug_assert!(self.stacks.len() > 0);
        self.stacks.last_mut().unwrap()
    }
}

#[cfg(test)]
mod tests {
    mod val {
        use super::super::*;

        #[test]
        fn val_to_u8_works() {
            let val = Val::Byte(15);
            assert_eq!(val.to_u8(), 15);

            let val = Val::Int(54);
            assert_eq!(val.to_u8(), 54);
        }

        #[test]
        fn val_to_i64_works() {
            let val = Val::Byte(15);
            assert_eq!(val.to_i64(), 15);

            let val = Val::Int(54);
            assert_eq!(val.to_i64(), 54)
        }

        #[test]
        fn val_to_f64_works() {
            let val = Val::Byte(15);
            assert_eq!(val.to_f64(), 15.0);
        }
    }

    mod val_stack {
        use super::super::*;

        #[test]
        fn new_works() {
            let stack = ValStack::new();

            assert_eq!(stack.len(), 0);
            assert_eq!(stack.register, None);
        }

        #[test]
        fn push_works() {
            let mut stack = ValStack::new();
            stack.push(Val::Byte(5));
            stack.push(Val::Int(42));
            stack.push(Val::Float(5.8));

            assert_eq!(stack.len(), 3);
            assert_eq!(stack.values, vec![Val::Byte(5), Val::Int(42), Val::Float(5.8)]);
        }

        #[test]
        fn has_at_least_works() {
            let mut stack = ValStack::new();
            stack.push(Val::Byte(5));
            stack.push(Val::Int(42));
            stack.push(Val::Float(5.8));

            assert!(stack.has_at_least(1));
            assert!(stack.has_at_least(2));
            assert!(stack.has_at_least(3));
            assert!(!stack.has_at_least(4));
        }

        #[test]
        fn pop_works() {
            let mut stack = ValStack::new();
            stack.push(Val::Byte(5));

            assert_eq!(stack.pop(), Some(Val::Byte(5)));
        }

        #[test]
        fn pop_empty_stack_returns_none() {
            let mut stack = ValStack::new();
            assert_eq!(stack.pop(), None);
        }

        #[test]
        fn set_register_value_works() {
            let mut stack = ValStack::new();
            assert_eq!(stack.register, None);

            stack.set_register_value(Val::Byte(12));
            assert_eq!(stack.register, Some(Val::Byte(12)));
        }

        #[test]
        fn clear_register_works() {
            let mut stack = ValStack::new();
            assert_eq!(stack.register, None);

            stack.set_register_value(Val::Byte(12));
            stack.clear_register();

            assert_eq!(stack.register, None);
        }
    }

    mod stack_of_stacks {
        use super::super::*;

        #[test]
        fn new_works() {
            let s = StackOfStacks::new();

            assert_eq!(s.stacks.len(), 1);
            assert_eq!(s.stacks[0].len(), 0);
            assert_eq!(s.stacks[0].register, None);
        }

        #[test]
        fn push_stack_works() {
            let mut s = StackOfStacks::new();
            s.top().push(Val::Byte(5));
            s.top().push(Val::Int(42));
            s.top().push(Val::Float(5.8));

            let res = s.push_stack(2);

            assert!(res.is_ok());

            assert_eq!(s.stacks.len(), 2);
            assert_eq!(s.stacks[0].values, vec![Val::Byte(5)]);
            assert_eq!(s.stacks[1].values, vec![Val::Int(42), Val::Float(5.8)]);
            assert_eq!(s.stacks[1].register, None);
        }

        #[test]
        fn push_stack_with_all_elements_works() {
            let mut s = StackOfStacks::new();
            s.top().push(Val::Byte(5));
            s.top().push(Val::Int(42));
            s.top().push(Val::Float(5.8));

            let res = s.push_stack(3);

            assert!(res.is_ok());

            assert_eq!(s.stacks.len(), 2);
            assert_eq!(s.stacks[0].values, vec![]);
            assert_eq!(s.stacks[1].values, vec![Val::Byte(5), Val::Int(42), Val::Float(5.8)]);
            assert_eq!(s.stacks[1].register, None);
        }

        #[test]
        fn push_stack_with_zero_elements_works() {
            let mut s = StackOfStacks::new();
            s.top().push(Val::Byte(5));
            s.top().push(Val::Int(42));
            s.top().push(Val::Float(5.8));

            let res = s.push_stack(0);

            assert!(res.is_ok());

            assert_eq!(s.stacks.len(), 2);
            assert_eq!(s.stacks[0].values, vec![Val::Byte(5), Val::Int(42), Val::Float(5.8)]);
            assert_eq!(s.stacks[1].values, vec![]);
            assert_eq!(s.stacks[1].register, None);
        }

        #[test]
        fn push_stack_with_too_many_values_fails() {
            let mut s = StackOfStacks::new();
            s.top().push(Val::Byte(5));
            s.top().push(Val::Int(42));

            let res = s.push_stack(3);

            assert!(res.is_err());
        }

        #[test]
        fn pop_stack_works() {
            let mut s = StackOfStacks::new();
            s.top().push(Val::Byte(5));
            s.top().push(Val::Int(42));
            s.top().push(Val::Float(5.8));

            let _ = s.push_stack(2).unwrap();

            s.pop_stack();

            assert_eq!(s.stacks.len(), 1);
            assert_eq!(s.stacks[0].values, vec![Val::Byte(5), Val::Int(42), Val::Float(5.8)]);
            assert_eq!(s.stacks[0].register, None);
        }

        #[test]
        fn pop_stack_with_base_register_works() {
            let mut s = StackOfStacks::new();
            s.top().push(Val::Byte(5));
            s.top().push(Val::Int(42));
            s.top().push(Val::Float(5.8));

            let _ = s.switch_register().unwrap();
            let _ = s.push_stack(1).unwrap();

            s.pop_stack();

            assert_eq!(s.stacks.len(), 1);
            assert_eq!(s.stacks[0].values, vec![Val::Byte(5), Val::Int(42)]);
            assert_eq!(s.stacks[0].register, Some(Val::Float(5.8)));
        }

        #[test]
        fn pop_stack_with_top_register_works() {
            let mut s = StackOfStacks::new();
            s.top().push(Val::Byte(5));
            s.top().push(Val::Int(42));
            s.top().push(Val::Float(5.8));

            let _ = s.push_stack(2).unwrap();
            let _ = s.switch_register().unwrap();

            s.pop_stack();

            assert_eq!(s.stacks.len(), 1);
            assert_eq!(s.stacks[0].values, vec![Val::Byte(5), Val::Int(42)]);
            assert_eq!(s.stacks[0].register, None);
        }

        #[test]
        fn pop_last_stack_makes_it_empty() {
            let mut s = StackOfStacks::new();
            s.top().push(Val::Byte(5));
            s.top().push(Val::Int(42));
            s.top().push(Val::Float(5.8));

            let _ = s.switch_register().unwrap();

            s.pop_stack();

            assert_eq!(s.stacks.len(), 1);
            assert_eq!(s.stacks[0].values, vec![]);
            assert_eq!(s.stacks[0].register, None);
        }

        #[test]
        fn switch_register_works() {
            let mut s = StackOfStacks::new();
            s.top().push(Val::Byte(5));
            s.top().push(Val::Int(42));
            s.top().push(Val::Float(5.8));

            let res = s.switch_register();

            assert!(res.is_ok());
            assert_eq!(s.stacks[0].register, Some(Val::Float(5.8)));
            assert_eq!(s.stacks[0].values, vec![Val::Byte(5), Val::Int(42)]);

            let res2 = s.switch_register();

            assert!(res2.is_ok());
            assert_eq!(s.stacks[0].register, None);
            assert_eq!(s.stacks[0].values, vec![Val::Byte(5), Val::Int(42), Val::Float(5.8)]);
        }

        #[test]
        fn switch_empty_register_on_empty_stack_fails() {
            let mut s = StackOfStacks::new();

            let res = s.switch_register();

            assert!(res.is_err());
        }
    }
}
