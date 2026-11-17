use crate::{Memory, kernel::Syscall};

impl<M: Memory> Syscall<'_, M> {
    pub fn stdin(&mut self) {
        self.set_result(0);
    }

    pub fn stdout(&mut self) {
        self.set_result(1);
    }

    pub fn stderr(&mut self) {
        self.set_result(2);
    }
}
