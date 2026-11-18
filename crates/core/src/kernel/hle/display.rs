use crate::{Memory, kernel::Syscall, mmio::addr::Address};

impl<M: Memory> Syscall<'_, M> {
    pub fn set_display_mode(&mut self) {
        tracing::info!(
            mode = self.arg(0),
            width = self.arg(1),
            height = self.arg(2),
            "display mode set"
        );
        self.set_result(0);
    }

    pub fn set_frame_buffer(&mut self) {
        self.display.framebuffer_addr = Address(self.arg(0));
        self.display.stride = self.arg(1);
        tracing::info!(%self.display.framebuffer_addr, self.display.stride, "frame buffer set");
        self.set_result(0);
    }
}
