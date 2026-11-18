mod hle;
mod nids;
mod syscalls;
mod threads;

use std::collections::BTreeMap;

pub(crate) use syscalls::Syscall;

use threads::{Callback, ROOT_THREAD, ThreadManager};

pub(crate) const INITIAL_SP: u32 = 0x09FF_FE90;

pub(crate) const MODULE_RETURN_STUB: u32 = 0x0800_0000;
pub(crate) const MODULE_RETURN_SYSCALL: u32 = 0x6767;
const IMPORT_SYSCALL_BASE: u32 = MODULE_RETURN_SYSCALL + 1;

pub(crate) const SYSCALL: u32 = 0x0000_000C;

pub(crate) struct Kernel {
    pub(crate) threads: ThreadManager,
    callbacks: BTreeMap<u32, Callback>,
    exit_callback: Option<u32>,
    imports: Vec<u32>,
    next_handle: u32,
}

impl Kernel {
    pub(crate) fn new() -> Self {
        Self {
            threads: ThreadManager::new(),
            callbacks: BTreeMap::new(),
            exit_callback: None,
            imports: Vec::new(),
            next_handle: ROOT_THREAD + 1,
        }
    }

    fn allocate_handle(&mut self) -> u32 {
        let handle = self.next_handle;
        self.next_handle += 1;
        handle
    }
}
