mod syscalls;
mod threads;

use threads::ThreadManager;

pub(crate) const MODULE_RETURN_SYSCALL: u32 = 0x6767;

pub(crate) struct Kernel {
    pub(crate) threads: ThreadManager,
}

impl Kernel {
    pub(crate) fn new() -> Self {
        Self {
            threads: ThreadManager::new(),
        }
    }
}
