pub trait Command {
    fn execute(&mut self);
    fn undo(&mut self);
}
pub struct CommandManager {
    undo_stack: Vec<Box<dyn Command>>,
    redo_stack: Vec<Box<dyn Command>>,
}

impl CommandManager {
    pub fn new() -> Self {
        Self { undo_stack: vec![], redo_stack: vec![] }
    }

    pub fn execute(&mut self, mut cmd: Box<dyn Command>) {
        cmd.execute();
        self.undo_stack.push(cmd);
        self.redo_stack.clear();
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn undo(&mut self) -> bool {
        if !self.can_undo() {
            return false;
        }

        let mut cmd = self.undo_stack.pop().unwrap();
        cmd.undo();
        self.redo_stack.push(cmd);

        true
    }

    pub fn redo(&mut self) -> bool {
        if !self.can_redo() {
            return false;
        }

        let mut cmd = self.redo_stack.pop().unwrap();
        cmd.execute();
        self.undo_stack.push(cmd);

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    struct AddCommand {
        target: Rc<RefCell<i32>>,
        amount: i32,
    }

    impl Command for AddCommand {
        fn execute(&mut self) {
            *self.target.borrow_mut() += self.amount;
        }

        fn undo(&mut self) {
            *self.target.borrow_mut() -= self.amount;
        }
    }

    #[test]
    fn test_command_manager() {
        let value = Rc::new(RefCell::new(10));
        let cmd = Box::new(AddCommand { target: Rc::clone(&value), amount: 5 });

        let mut mgr = CommandManager::new();
        mgr.execute(cmd); // value = 15
        assert_eq!(*value.borrow(), 15);

        mgr.undo(); // value = 10
        assert_eq!(*value.borrow(), 10);

        mgr.redo(); // value = 15 again
        assert_eq!(*value.borrow(), 15);
    }
}
