#[derive(PartialEq, Debug)]
pub enum Error {
    StackUnderflow,
}

pub struct Stack<T> {
    pub values: Vec<T>,
    pub register: Option<T>,
}

impl<T> Default for Stack<T>
where
    T: Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Stack<T>
where
    T: Clone,
{
    pub fn new() -> Self {
        Stack {
            values: Vec::new(),
            register: None,
        }
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn push(&mut self, val: T) {
        self.values.push(val);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.values.pop()
    }

    pub fn switch_register(&mut self) -> Result<(), Error> {
        match self.register.take() {
            Some(val) => self.push(val),
            None => self.register = Some(self.pop().ok_or(Error::StackUnderflow)?),
        }

        Ok(())
    }

    pub fn dup(&mut self) -> Result<(), Error> {
        let v = self.values.last().ok_or(Error::StackUnderflow)?.clone();
        self.values.push(v);
        Ok(())
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
            0 | 1 => Err(Error::StackUnderflow),
            n => {
                self.values.swap(n - 2, n - 1);
                Ok(())
            }
        }
    }

    pub fn swap2(&mut self) -> Result<(), Error> {
        match self.values.len() {
            0..=2 => Err(Error::StackUnderflow),
            n => {
                self.values.swap(n - 2, n - 1);
                self.values.swap(n - 3, n - 2);
                Ok(())
            }
        }
    }

    pub fn rshift(&mut self) {
        let e = self.values.pop();
        if let Some(e) = e {
            self.values.insert(0, e);
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

pub struct StackOfStacks<T> {
    pub initial_stack: Stack<T>,
    pub additional_stacks: Vec<Stack<T>>,
}

impl<T> Default for StackOfStacks<T>
where
    T: Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> StackOfStacks<T>
where
    T: Clone,
{
    pub fn new() -> Self {
        StackOfStacks {
            initial_stack: Stack::new(),
            additional_stacks: vec![],
        }
    }

    pub fn push_stack(&mut self, moved_items: usize) -> Result<(), Error> {
        let vals = {
            let stack = self.top_mut();
            let n = stack
                .len()
                .checked_sub(moved_items)
                .ok_or(Error::StackUnderflow)?;
            stack.values.split_off(n)
        };

        self.additional_stacks.push(Stack {
            values: vals,
            register: None,
        });

        Ok(())
    }

    pub fn pop_stack(&mut self) {
        if let Some(stack) = self.additional_stacks.pop() {
            self.top_mut().values.extend(stack.values);
        } else {
            self.initial_stack.values.clear();
            self.initial_stack.register = None;
        }
    }

    pub fn top(&self) -> &Stack<T> {
        self.additional_stacks.last().unwrap_or(&self.initial_stack)
    }

    pub fn top_mut(&mut self) -> &mut Stack<T> {
        self.additional_stacks
            .last_mut()
            .unwrap_or(&mut self.initial_stack)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_works() {
        let stack = Stack::<isize>::new();

        assert!(stack.is_empty());
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

#[cfg(test)]
mod stack_of_stacks_tests {
    use super::*;

    #[test]
    fn new_works() {
        let s = StackOfStacks::<isize>::new();

        assert_eq!(s.additional_stacks.len(), 0);
        assert_eq!(s.initial_stack.len(), 0);
        assert_eq!(s.initial_stack.register, None);
    }

    #[test]
    fn push_stack_works() {
        let mut s = StackOfStacks::new();
        s.top_mut().push(5);
        s.top_mut().push(42);
        s.top_mut().push(58);

        let res = s.push_stack(2);

        assert!(res.is_ok());

        assert_eq!(s.additional_stacks.len(), 1);
        assert_eq!(s.initial_stack.values, vec![5]);
        assert_eq!(s.additional_stacks[0].values, vec![42, 58]);
        assert_eq!(s.additional_stacks[0].register, None);
    }

    #[test]
    fn push_stack_with_all_elements_works() {
        let mut s = StackOfStacks::new();
        s.top_mut().push(5);
        s.top_mut().push(42);
        s.top_mut().push(58);

        let res = s.push_stack(3);

        assert!(res.is_ok());

        assert_eq!(s.additional_stacks.len(), 1);
        assert_eq!(s.initial_stack.values, vec![0; 0]);
        assert_eq!(s.additional_stacks[0].values, vec![5, 42, 58]);
        assert_eq!(s.additional_stacks[0].register, None);
    }

    #[test]
    fn push_stack_with_zero_elements_works() {
        let mut s = StackOfStacks::new();
        s.top_mut().push(5);
        s.top_mut().push(42);
        s.top_mut().push(58);

        let res = s.push_stack(0);

        assert!(res.is_ok());

        assert_eq!(s.additional_stacks.len(), 1);
        assert_eq!(s.initial_stack.values, vec![5, 42, 58]);
        assert_eq!(s.additional_stacks[0].values, vec![0; 0]);
        assert_eq!(s.additional_stacks[0].register, None);
    }

    #[test]
    fn push_stack_with_too_many_values_fails() {
        let mut s = StackOfStacks::new();
        s.top_mut().push(5);
        s.top_mut().push(42);

        let res = s.push_stack(3);

        assert_eq!(res, Err(Error::StackUnderflow));
    }

    #[test]
    fn pop_stack_works() {
        let mut s = StackOfStacks::new();
        s.top_mut().push(5);
        s.top_mut().push(42);
        s.top_mut().push(58);

        let res = s.push_stack(2);

        assert!(res.is_ok());

        s.pop_stack();

        assert_eq!(s.additional_stacks.len(), 0);
        assert_eq!(s.initial_stack.values, vec![5, 42, 58]);
        assert_eq!(s.initial_stack.register, None);
    }

    #[test]
    fn pop_stack_with_base_register_works() {
        let mut s = StackOfStacks::new();
        s.top_mut().push(5);
        s.top_mut().push(42);
        s.top_mut().push(58);

        let _ = s.top_mut().switch_register().unwrap();
        let _ = s.push_stack(1).unwrap();

        s.pop_stack();

        assert_eq!(s.additional_stacks.len(), 0);
        assert_eq!(s.initial_stack.values, vec![5, 42]);
        assert_eq!(s.initial_stack.register, Some(58));
    }

    #[test]
    fn pop_stack_with_top_register_works() {
        let mut s = StackOfStacks::new();
        s.top_mut().push(5);
        s.top_mut().push(42);
        s.top_mut().push(58);

        let _ = s.push_stack(2).unwrap();
        let _ = s.top_mut().switch_register().unwrap();

        s.pop_stack();

        assert_eq!(s.additional_stacks.len(), 0);
        assert_eq!(s.initial_stack.values, vec![5, 42]);
        assert_eq!(s.initial_stack.register, None);
    }

    #[test]
    fn pop_initial_stack_makes_it_empty() {
        let mut s = StackOfStacks::new();
        s.top_mut().push(5);
        s.top_mut().push(42);
        s.top_mut().push(58);

        let _ = s.top_mut().switch_register().unwrap();

        s.pop_stack();

        assert_eq!(s.additional_stacks.len(), 0);
        assert_eq!(s.initial_stack.values, vec![0; 0]);
        assert_eq!(s.initial_stack.register, None);
    }
}
