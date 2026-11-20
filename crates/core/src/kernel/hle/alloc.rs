use crate::{Memory, kernel::Syscall};

impl<M: Memory> Syscall<'_, M> {
    pub fn alloc_partition_memory(&mut self) {
        let handle = self.kernel.allocate_handle();
        self.kernel
            .memory
            .alloc(handle, self.arg(3) as usize, &self.arg_as_str(1));
        self.set_result(handle);
    }

    pub fn get_block_address(&mut self) {
        let addr = self.kernel.memory.address_of(self.arg(0));
        self.set_result(addr.into());
    }
}
