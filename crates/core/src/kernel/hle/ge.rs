use crate::{Memory, kernel::Syscall, mmio::VRAM_ADDR};

impl<M: Memory> Syscall<'_, M> {
    pub fn get_edram_addr(&mut self) {
        self.set_result(VRAM_ADDR);
    }
}
