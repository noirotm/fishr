#[derive(PartialEq, Debug)]
pub enum Error {
    StackUnderflow,
}

pub struct Stack<T>
    where T: Copy
{
    pub values: Vec<T>,
    pub register: Option<T>,
}

impl<T> Stack<T>
    where T: Copy
{
    pub fn new() -> Stack<T> {
        Stack {
            values: Vec::new(),
            register: None,
        }
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn push(&mut self, val: T) {
        self.values.push(val);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.values.pop()
    }

    pub fn switch_register(&mut self) -> Result<(), Error> {
        match self.register.take() {
            None => {
                match self.pop() {
                    Some(val) => self.register = Some(val),
                    None => return Err(Error::StackUnderflow),
                }
            }
            Some(val) => self.push(val),
        }

        Ok(())
    }

    pub fn dup(&mut self) -> Result<(), Error> {
        match self.values.len() {
            0 => Err(Error::StackUnderflow),
            n => {
                let v = self.values[n - 1];
                self.values.push(v);
                Ok(())
            }
        }
    }

    pub fn drop(&mut self) -> Result<(), Error> {
        match self.values.len() {
            0 => Err(Error::StackUnderflow),
            n => {
                self.values.truncate(n - 1);
                Ok(())
            }
        }
    }

    pub fn swap(&mut self) -> Result<(), Error> {
        match self.values.len() {
            n if n >= 2 => {
                let x = self.values[n - 1];
                let y = self.values[n - 2];
                self.values[n - 2] = x;
                self.values[n - 1] = y;
                Ok(())
            }
            _ => Err(Error::StackUnderflow),
        }
    }

    pub fn swap2(&mut self) -> Result<(), Error> {
        match self.values.len() {
            n if n >= 3 => {
                let x = self.values[n - 3];
                let y = self.values[n - 2];
                let z = self.values[n - 1];
                self.values[n - 3] = z;
                self.values[n - 2] = x;
                self.values[n - 1] = y;
                Ok(())
            }
            _ => Err(Error::StackUnderflow),
        }
    }

    pub fn rshift(&mut self) {
        match self.values.len() {
            0 | 1 => {}
            n => {
                let mut v: Vec<_> = self.values.drain(0..n - 1).collect();
                self.values.append(&mut v);
            }
        }
    }

    pub fn lshift(&mut self) {
        match self.values.len() {
            0 | 1 => {}
            _ => {
                let v = self.values.remove(0);
                self.values.push(v);
            }
        }
    }
}

pub struct StackOfStacks<T>
    where T: Copy
{
    pub stacks: Vec<Stack<T>>,
}

impl<T> StackOfStacks<T>
    where T: Copy
{
    pub fn new() -> StackOfStacks<T> {
        StackOfStacks { stacks: vec![Stack::<T>::new()] /* there is always at least one stack */ }
    }

    pub fn push_stack(&mut self, moved_items: usize) -> Result<(), Error> {
        let vals: Vec<_> = {
            let stack = self.top();
            match stack.len().checked_sub(moved_items) {
                Some(v) => stack.values.split_off(v),
                None => return Err(Error::StackUnderflow),
            }
        };

        self.stacks.push(Stack::<T> {
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

    pub fn top(&mut self) -> &mut Stack<T> {
        debug_assert!(self.stacks.len() > 0);
        self.stacks.last_mut().unwrap()
    }
}

#[cfg(test)]
mod tests {
    mod stack {
        use super::super::*;

        #[test]
        fn new_works() {
            let stack = Stack::<isize>::new();

            assert_eq!(stack.len(), 0);
            assert_eq!(stack.register, None);
        }

        #[test]
        fn push_works() {
            let mut stack = Stack::new();
            stack.push(5);
            stack.push(42);
            stack.push(58);

            assert_eq!(stack.len(), 3);
            assert_eq!(stack.values, vec![5, 42, 58]);
        }

        #[test]
        fn pop_works() {
            let mut stack = Stack::<isize>::new();
            stack.push(5);

            assert_eq!(stack.pop(), Some(5));
        }

        #[test]
        fn pop_empty_stack_returns_none() {
            let mut stack = Stack::<isize>::new();
            assert_eq!(stack.pop(), None);
        }

        #[test]
        fn dup_works() {
            let mut stack = Stack::new();
            stack.push(5);
            stack.push(42);

            let res = stack.dup();

            assert!(res.is_ok());
            assert_eq!(stack.values, vec![5, 42, 42]);
        }

        #[test]
        fn dup_with_empty_stack_fails() {
            let mut stack = Stack::<isize>::new();

            let res = stack.dup();

            assert_eq!(res, Err(Error::StackUnderflow));
        }

        #[test]
        fn drop_works() {
            let mut stack = Stack::new();
            stack.push(5);
            stack.push(42);

            let res = stack.drop();

            assert!(res.is_ok());
            assert_eq!(stack.values, vec![5]);
        }

        #[test]
        fn drop_with_empty_stack_fails() {
            let mut stack = Stack::<isize>::new();

            let res = stack.drop();

            assert_eq!(res, Err(Error::StackUnderflow));
        }

        #[test]
        fn switch_register_works() {
            let mut stack = Stack::new();
            stack.push(5);
            stack.push(42);
            stack.push(58);

            let res = stack.switch_register();

            assert!(res.is_ok());
            assert_eq!(stack.register, Some(58));
            assert_eq!(stack.values, vec![5, 42]);

            let res2 = stack.switch_register();

            assert!(res2.is_ok());
            assert_eq!(stack.register, None);
            assert_eq!(stack.values, vec![5, 42, 58]);
        }

        #[test]
        fn switch_empty_register_on_empty_stack_fails() {
            let mut stack = Stack::<isize>::new();

            let res = stack.switch_register();

            assert_eq!(res, Err(Error::StackUnderflow));
        }

        #[test]
        fn swap_works() {
            let mut stack = Stack::new();
            stack.push(1);
            stack.push(2);
            stack.push(3);

            let res = stack.swap();

            assert!(res.is_ok());
            assert_eq!(stack.values, vec![1, 3, 2]);
        }

        #[test]
        fn swap_with_empty_stack_fails() {
            let mut stack = Stack::<isize>::new();

            let res = stack.swap();

            assert_eq!(res, Err(Error::StackUnderflow));
        }

        #[test]
        fn swap_with_one_element_fails() {
            let mut stack = Stack::new();
            stack.push(1);

            let res = stack.swap();

            assert_eq!(res, Err(Error::StackUnderflow));
        }

        #[test]
        fn swap2_works() {
            let mut stack = Stack::new();
            stack.push(1);
            stack.push(2);
            stack.push(3);
            stack.push(4);

            let res = stack.swap2();

            assert!(res.is_ok());
            assert_eq!(stack.values, vec![1, 4, 2, 3]);
        }

        #[test]
        fn swap2_with_empty_stack_fails() {
            let mut stack = Stack::<isize>::new();

            let res = stack.swap2();

            assert_eq!(res, Err(Error::StackUnderflow));
        }

        #[test]
        fn swap2_with_one_element_fails() {
            let mut stack = Stack::new();
            stack.push(1);

            let res = stack.swap2();

            assert_eq!(res, Err(Error::StackUnderflow));
        }

        #[test]
        fn swap2_with_two_elements_fails() {
            let mut stack = Stack::new();
            stack.push(1);
            stack.push(2);

            let res = stack.swap2();

            assert_eq!(res, Err(Error::StackUnderflow));
        }

        #[test]
        fn rshift_works() {
            let mut stack = Stack::new();
            stack.push(1);
            stack.push(2);
            stack.push(3);
            stack.push(4);

            stack.rshift();

            assert_eq!(stack.values, vec![4, 1, 2, 3]);
        }

        #[test]
        fn lshift_works() {
            let mut stack = Stack::new();
            stack.push(1);
            stack.push(2);
            stack.push(3);
            stack.push(4);

            stack.lshift();

            assert_eq!(stack.values, vec![2, 3, 4, 1]);
        }
    }

    mod stack_of_stacks {
        use super::super::*;

        #[test]
        fn new_works() {
            let s = StackOfStacks::<isize>::new();

            assert_eq!(s.stacks.len(), 1);
            assert_eq!(s.stacks[0].len(), 0);
            assert_eq!(s.stacks[0].register, None);
        }

        #[test]
        fn push_stack_works() {
            let mut s = StackOfStacks::new();
            s.top().push(5);
            s.top().push(42);
            s.top().push(58);

            let res = s.push_stack(2);

            assert!(res.is_ok());

            assert_eq!(s.stacks.len(), 2);
            assert_eq!(s.stacks[0].values, vec![5]);
            assert_eq!(s.stacks[1].values, vec![42, 58]);
            assert_eq!(s.stacks[1].register, None);
        }

        #[test]
        fn push_stack_with_all_elements_works() {
            let mut s = StackOfStacks::new();
            s.top().push(5);
            s.top().push(42);
            s.top().push(58);

            let res = s.push_stack(3);

            assert!(res.is_ok());

            assert_eq!(s.stacks.len(), 2);
            assert_eq!(s.stacks[0].values, vec![0; 0]);
            assert_eq!(s.stacks[1].values, vec![5, 42, 58]);
            assert_eq!(s.stacks[1].register, None);
        }

        #[test]
        fn push_stack_with_zero_elements_works() {
            let mut s = StackOfStacks::new();
            s.top().push(5);
            s.top().push(42);
            s.top().push(58);

            let res = s.push_stack(0);

            assert!(res.is_ok());

            assert_eq!(s.stacks.len(), 2);
            assert_eq!(s.stacks[0].values, vec![5, 42, 58]);
            assert_eq!(s.stacks[1].values, vec![0; 0]);
            assert_eq!(s.stacks[1].register, None);
        }

        #[test]
        fn push_stack_with_too_many_values_fails() {
            let mut s = StackOfStacks::new();
            s.top().push(5);
            s.top().push(42);

            let res = s.push_stack(3);

            assert_eq!(res, Err(Error::StackUnderflow));
        }

        #[test]
        fn pop_stack_works() {
            let mut s = StackOfStacks::new();
            s.top().push(5);
            s.top().push(42);
            s.top().push(58);

            let res = s.push_stack(2);

            assert!(res.is_ok());

            s.pop_stack();

            assert_eq!(s.stacks.len(), 1);
            assert_eq!(s.stacks[0].values, vec![5, 42, 58]);
            assert_eq!(s.stacks[0].register, None);
        }

        #[test]
        fn pop_stack_with_base_register_works() {
            let mut s = StackOfStacks::new();
            s.top().push(5);
            s.top().push(42);
            s.top().push(58);

            let _ = s.top().switch_register().unwrap();
            let _ = s.push_stack(1).unwrap();

            s.pop_stack();

            assert_eq!(s.stacks.len(), 1);
            assert_eq!(s.stacks[0].values, vec![5, 42]);
            assert_eq!(s.stacks[0].register, Some(58));
        }

        #[test]
        fn pop_stack_with_top_register_works() {
            let mut s = StackOfStacks::new();
            s.top().push(5);
            s.top().push(42);
            s.top().push(58);

            let _ = s.push_stack(2).unwrap();
            let _ = s.top().switch_register().unwrap();

            s.pop_stack();

            assert_eq!(s.stacks.len(), 1);
            assert_eq!(s.stacks[0].values, vec![5, 42]);
            assert_eq!(s.stacks[0].register, None);
        }

        #[test]
        fn pop_last_stack_makes_it_empty() {
            let mut s = StackOfStacks::new();
            s.top().push(5);
            s.top().push(42);
            s.top().push(58);

            let _ = s.top().switch_register().unwrap();

            s.pop_stack();

            assert_eq!(s.stacks.len(), 1);
            assert_eq!(s.stacks[0].values, vec![0; 0]);
            assert_eq!(s.stacks[0].register, None);
        }
    }
}
