use crate::{Memory, kernel::Syscall};

impl<M: Memory> Syscall<'_, M> {
    pub fn device_control(&mut self) {
        tracing::error!(
            device = self.arg_as_str(0),
            command = format_args!("{:08X}", self.arg(1)),
            "devctl called"
        );
        self.set_result(0);
    }
}
