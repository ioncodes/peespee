use std::ffi::CStr;

use crate::Memory;
use crate::cpu::{REG_A0, REG_A1, REG_SP};
use crate::kernel::Syscall;

const STACK_ALIGNMENT: u32 = 16;

impl<M: Memory> Syscall<'_, M> {
    pub fn create_thread(&mut self) {
        let handle = self.kernel.threads.create_thread(
            self.thread_name(),
            self.arg(1),
            self.arg(2) as u8,
            self.arg(3),
        );
        self.set_result(handle);
    }

    pub fn start_thread(&mut self) {
        let handle = self.arg(0);
        let arg_size = self.arg(1);
        let arg_addr = self.arg(2);

        let mut arguments = vec![0; arg_size as usize];
        self.mmio.read(arg_addr, &mut arguments);

        let thread = self.kernel.threads.find_thread_mut(handle);
        let sp = (thread.context.registers[REG_SP as usize] - arg_size) & !(STACK_ALIGNMENT - 1);
        self.mmio.write(sp, &arguments);

        thread.context.registers[REG_A0 as usize] = arg_size;
        thread.context.registers[REG_A1 as usize] = sp;
        thread.context.registers[REG_SP as usize] = sp;
        thread.ready = true;
        tracing::info!(thread = %thread.name, "thread started");

        self.set_result(0);
    }

    fn thread_name(&self) -> String {
        let mut buffer = [0u8; 16];
        self.mmio.read(self.arg(0), &mut buffer);
        CStr::from_bytes_until_nul(&buffer)
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned()
    }
}
