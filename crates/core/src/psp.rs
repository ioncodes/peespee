use crate::cpu::{Outcome, REG_RA, REG_SP};
use crate::format;
use crate::kernel::{
    INITIAL_SP, Kernel, MODULE_RETURN_STUB, MODULE_RETURN_SYSCALL, SYSCALL, Syscall,
};
use crate::{Cpu, Display, Ge, Memory, Mmio};

const INSTRUCTIONS_PER_FRAME: u32 = 333_000_000 / 60;

pub struct Psp {
    mmio: Mmio,
    cpu: Cpu,
    kernel: Kernel,
    display: Display,
    ge: Ge,
}

impl Psp {
    pub fn new(buffer: &[u8]) -> Self {
        let mut mmio = Mmio::new();
        let mut kernel = Kernel::new();

        let prx = format::executable(buffer);
        let base = prx.load_base(0x0880_4000);
        let pc = prx.load_into(&mut mmio, base);
        kernel.link_imports(&prx, base, &mut mmio);

        // Return from the module entry point through an internal kernel syscall.
        let mut cpu = Cpu::new(pc.into());
        cpu.set_register(REG_SP, INITIAL_SP);
        cpu.set_register(REG_RA, MODULE_RETURN_STUB);
        mmio.write32(MODULE_RETURN_STUB, (MODULE_RETURN_SYSCALL << 6) | SYSCALL);

        Self {
            mmio,
            cpu,
            kernel,
            display: Display::new(),
            ge: Ge,
        }
    }

    pub fn run_frame(&mut self, framebuffer: &mut [u8]) {
        for _ in 0..INSTRUCTIONS_PER_FRAME {
            if !self.kernel.threads.has_active_threads() {
                break;
            }

            match self.cpu.step(&mut self.mmio) {
                Outcome::Continue => {}
                Outcome::Syscall(code) => Syscall {
                    kernel: &mut self.kernel,
                    cpu: &mut self.cpu,
                    mmio: &mut self.mmio,
                    display: &mut self.display,
                }
                .dispatch(code),
            }
        }

        self.display.render_framebuffer(&self.mmio, framebuffer);
    }
}
