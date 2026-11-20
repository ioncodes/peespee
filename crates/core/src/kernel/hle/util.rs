use crate::{Memory, kernel::Syscall};

impl<M: Memory> Syscall<'_, M> {
    // TODO: various shit like x/o assignment for confirm/cancel
    pub fn get_system_parameter(&mut self) {
        tracing::error!("unknown system parameter: {}", self.arg(0));
        self.set_result(0);
    }
}
