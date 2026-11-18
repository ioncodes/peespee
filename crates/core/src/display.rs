use crate::Memory;
use crate::mmio::addr::Address;

pub const DISPLAY_WIDTH: u32 = 480;
pub const DISPLAY_HEIGHT: u32 = 272;

pub struct Display {
    pub(crate) framebuffer_addr: Address,
    pub(crate) stride: u32,
}

impl Display {
    pub fn new() -> Self {
        Self {
            framebuffer_addr: Address(0),
            stride: 512,
        }
    }

    pub fn render_framebuffer(&self, memory: &impl Memory, framebuffer: &mut [u8]) {
        let row_size = (DISPLAY_WIDTH * 4) as usize;
        for (y, row) in framebuffer.chunks_exact_mut(row_size).enumerate() {
            let source = y * (self.stride * 4) as usize;
            memory.read(self.framebuffer_addr.wrapping_add(source), row);
        }
    }
}
