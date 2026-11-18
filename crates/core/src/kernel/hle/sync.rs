use crate::{Memory, kernel::Syscall};

impl<M: Memory> Syscall<'_, M> {
    // sceKernelCreateLwMutex => Lw = Leightweight = User
    pub fn create_user_mutex(&mut self) {
        tracing::error!(name = self.arg_as_str(1), "creating user mutex");
        self.set_result(0);
    }

    pub fn create_semaphore(&mut self) {
        tracing::error!(name = self.arg_as_str(0), "creating semaphore");
        self.set_result(0);
    }
}
