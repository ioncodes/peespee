mod nids;
mod syscalls;
mod threads;
mod hle;

use syscalls::Syscall;

use threads::ThreadManager;

pub(crate) const INITIAL_SP: u32 = 0x09FF_FE90;

pub(crate) const MODULE_RETURN_STUB: u32 = 0x0800_0000;
pub(crate) const MODULE_RETURN_SYSCALL: u32 = 0x6767;
const IMPORT_SYSCALL_BASE: u32 = MODULE_RETURN_SYSCALL + 1;

pub(crate) const SYSCALL: u32 = 0x0000_000C;

pub(crate) struct Kernel {
    pub(crate) threads: ThreadManager,
    imports: Vec<u32>,
}

impl Kernel {
    pub(crate) fn new() -> Self {
        Self {
            threads: ThreadManager::new(),
            imports: Vec::new(),
        }
    }
}
