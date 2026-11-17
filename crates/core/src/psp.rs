use crate::cpu::{Outcome, REG_RA, REG_SP};
use crate::format;
use crate::kernel::{INITIAL_SP, Kernel, MODULE_RETURN_STUB, MODULE_RETURN_SYSCALL, SYSCALL};
use crate::{Cpu, Memory, Mmio};

pub struct Psp {
    mmio: Mmio,
    cpu: Cpu,
    kernel: Kernel,
}

impl Psp {
    pub fn new(buffer: &[u8]) -> Self {
        let prx = format::executable(buffer);
        let mut mmio = Mmio::new();
        let mut kernel = Kernel::new();
        let base = prx.load_base(0x0880_4000);
        let pc = prx.load_into(&mut mmio, base);
        kernel.link_imports(&prx, base, &mut mmio);

        // Return from the module entry point through an internal kernel syscall.
        let mut cpu = Cpu::new(pc.into());
        cpu.set_register(REG_SP, INITIAL_SP);
        cpu.set_register(REG_RA, MODULE_RETURN_STUB);
        mmio.write32(MODULE_RETURN_STUB, (MODULE_RETURN_SYSCALL << 6) | SYSCALL);

        Self { mmio, cpu, kernel }
    }

    pub fn run(&mut self) {
        while self.kernel.threads.has_active_threads() {
            match self.cpu.step(&mut self.mmio) {
                Outcome::Continue => {}
                Outcome::Syscall(code) => {
                    self.kernel.dispatch(code, &mut self.cpu, &mut self.mmio);
                }
            }
        }
    }
}
