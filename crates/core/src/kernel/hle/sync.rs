use crate::{Memory, kernel::Syscall};

impl<M: Memory> Syscall<'_, M> {
    // sceKernelCreateLwMutex => Lw = Lightweight = User
    pub fn create_user_mutex(&mut self) {
        tracing::error!(name = self.arg_as_str(1), "creating user mutex");
        self.set_result(0);
    }

    pub fn lock_user_mutex(&mut self) {
        tracing::error!(
            work_area = format_args!("{:08X}", self.arg(0)),
            count = self.arg(1),
            "locking user mutex"
        );
        self.set_result(0);
    }

    pub fn unlock_user_mutex(&mut self) {
        tracing::error!(
            work_area = format_args!("{:08X}", self.arg(0)),
            count = self.arg(1),
            "unlocking user mutex"
        );
        self.set_result(0);
    }

    pub fn delete_user_mutex(&mut self) {
        tracing::error!(
            work_area = format_args!("{:08X}", self.arg(0)),
            "deleting user mutex"
        );
        self.set_result(0);
    }

    pub fn create_semaphore(&mut self) {
        tracing::error!(name = self.arg_as_str(0), "creating semaphore");
        self.set_result(0);
    }

    pub fn wait_semaphore(&mut self) {
        tracing::error!(
            id = self.arg(0),
            count = self.arg(1),
            "waiting on semaphore"
        );
        self.set_result(0);
    }

    pub fn signal_semaphore(&mut self) {
        tracing::error!(id = self.arg(0), count = self.arg(1), "signaling semaphore");
        self.set_result(0);
    }
}
