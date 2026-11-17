use crate::{Memory, kernel::Syscall};

impl<M: Memory> Syscall<'_, M> {
    pub fn set_display_mode(&mut self) {
        self.set_result(0);
    }

    pub fn set_frame_buffer(&mut self) {
        tracing::info!("frame buffer set");
        self.set_result(0);
    }
}
