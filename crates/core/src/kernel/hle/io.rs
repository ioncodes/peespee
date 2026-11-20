use crate::{Memory, kernel::Syscall};

impl<M: Memory> Syscall<'_, M> {
    pub fn write_io(&mut self) {
        tracing::error!(
            handle = self.arg(0),
            data = format_args!("{:08X}", self.arg(1)),
            size = self.arg(2),
            "sceIoWrite called"
        );
        self.set_result(self.arg(2));
    }
}
