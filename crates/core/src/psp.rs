use crate::cpu::{REG_RA, REG_SP, Outcome};
use crate::format::{Pbp, Prx};
use crate::kernel::{Kernel, MODULE_RETURN_SYSCALL};
use crate::mmio::addr::Address;
use crate::{Cpu, Memory, Mmio};

const INITIAL_SP: u32 = 0x09FFFE90;
const MODULE_RETURN_STUB: u32 = 0x0800_0000;

pub struct Psp<'a> {
    prx: Prx<'a>,
    mmio: Mmio,
    cpu: Cpu,
    kernel: Kernel,
}

impl<'a> Psp<'a> {
    pub fn new(buffer: &'a [u8]) -> Self {
        let prx = match crate::format::detect(buffer) {
            crate::format::Type::Pbp => Pbp::new(buffer).prx(),
            crate::format::Type::Prx => Prx::new(buffer).expect("failed to create Prx"),
        };
        let mut mmio = Mmio::new();
        let pc = prx
            .load_into(&mut mmio, Address(0x0880_4000))
            .expect("failed to load executable");

        // Return from the module entry point through an internal kernel syscall.
        let mut cpu = Cpu::new(pc.into());
        cpu.set_register(REG_SP, INITIAL_SP);
        cpu.set_register(REG_RA, MODULE_RETURN_STUB);
        mmio.write32(MODULE_RETURN_STUB, (MODULE_RETURN_SYSCALL << 6) | 0x0C);

        Self {
            prx,
            mmio,
            cpu,
            kernel: Kernel::new(),
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
