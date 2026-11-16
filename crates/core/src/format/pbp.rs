use super::Prx;

pub struct Pbp<'a> {
    buffer: &'a [u8],
}

impl<'a> Pbp<'a> {
    pub fn new(buffer: &'a [u8]) -> Self {
        Self { buffer }
    }

    pub fn prx(&self) -> Prx<'a> {
        let start_addr = u32::from_le_bytes(
            self.buffer[0x20..0x24]
                .try_into()
                .expect("failed to read addr"),
        );
        let end_addr = u32::from_le_bytes(
            self.buffer[0x24..0x28]
                .try_into()
                .expect("failed to read addr"),
        );
        Prx::new(&self.buffer[start_addr as usize..end_addr as usize])
            .expect("failed to create Prx")
    }
}