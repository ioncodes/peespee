use crate::{
    Memory,
    kernel::{Syscall, threads::Callback},
};

impl<M: Memory> Syscall<'_, M> {
    pub fn create_callback(&mut self) {
        let handle = self.kernel.allocate_handle();
        let callback = Callback {
            owner: self.kernel.threads.current_thread,
            entry: self.arg(1),
            common: self.arg(2),
        };

        self.kernel.callbacks.insert(handle, callback);
        self.set_result(handle);
    }

    pub fn register_exit_callback(&mut self) {
        self.kernel.exit_callback = Some(self.arg(0));
        self.set_result(0);
    }
}
