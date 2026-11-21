use crate::{Memory, kernel::Syscall};

impl<M: Memory> Syscall<'_, M> {
    pub fn open_io(&mut self) {
        tracing::error!(
            path = self.arg_as_str(0),
            flags = format_args!("{:08X}", self.arg(1)),
            "sceIoOpen called"
        );
        self.set_result(0);
    }

    pub fn write_io(&mut self) {
        tracing::error!(
            handle = self.arg(0),
            data = format_args!("{:08X}", self.arg(1)),
            size = self.arg(2),
            "sceIoWrite called"
        );
        self.set_result(self.arg(2));
    }

    pub fn device_control(&mut self) {
        let device = self.arg_as_str(0);
        tracing::error!(
            device,
            command = format_args!("{:08X}", self.arg(1)),
            "devctl called"
        );

        // TODO: This is pspautotests stuff
        if device == "emulator:" && self.arg(1) == 2 {
            let mut buffer = vec![0u8; self.arg(3) as usize];
            self.mmio.read(self.arg(2), buffer.as_mut_slice());
            print!("{}", String::from_utf8_lossy(&buffer));
        }

        self.set_result(0);
    }
}
