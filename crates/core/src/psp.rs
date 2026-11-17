use crate::cpu::{Outcome, REG_RA, REG_SP};
use crate::format::{self, Prx};
use crate::kernel::{Kernel, MODULE_RETURN_SYSCALL, SYSCALL};
use crate::mmio::addr::Address;
use crate::{Cpu, Memory, Mmio};

const INITIAL_SP: u32 = 0x09FFFE90;
const MODULE_RETURN_STUB: u32 = 0x0800_0000;

pub struct Psp {
    prx: Prx,
    mmio: Mmio,
    cpu: Cpu,
    kernel: Kernel,
}

impl Psp {
    pub fn new(buffer: &[u8]) -> Self {
        let prx = format::executable(buffer);
        let mut mmio = Mmio::new();
        let mut kernel = Kernel::new();
        let base = prx.load_base(Address(0x0880_4000));
        let pc = prx.load_into(&mut mmio, base);
        kernel.link_imports(&prx, base, &mut mmio);

        // Return from the module entry point through an internal kernel syscall.
        let mut cpu = Cpu::new(pc.into());
        cpu.set_register(REG_SP, INITIAL_SP);
        cpu.set_register(REG_RA, MODULE_RETURN_STUB);
        mmio.write32(MODULE_RETURN_STUB, (MODULE_RETURN_SYSCALL << 6) | SYSCALL);

        Self {
            prx,
            mmio,
            cpu,
            kernel,
        }
    }

    pub fn run(&mut self) {
        while self.kernel.threads.has_threads() {
            match self.cpu.step(&mut self.mmio) {
                Outcome::Continue => {}
                Outcome::Syscall(code) => {
                    self.kernel.dispatch(code, &mut self.cpu, &mut self.mmio);
                }
            }
        }
    }
}
