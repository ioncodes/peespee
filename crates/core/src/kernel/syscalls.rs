use crate::{Cpu, Memory};

use super::{Kernel, MODULE_RETURN_SYSCALL};

impl Kernel {
    pub(crate) fn dispatch(&mut self, syscall: u32, cpu: &mut Cpu, mmio: &mut impl Memory) {
        match syscall {
            MODULE_RETURN_SYSCALL => self.return_from_module(cpu, mmio),
            _ => todo!("Unhandled syscall: {syscall:08x}"),
        }
    }

    fn return_from_module(&mut self, cpu: &mut Cpu, _mmio: &mut impl Memory) {
        // The module entry point returns its status in $v0.
        self.threads.exit_current(cpu.register(2) as i32);
    }
}
