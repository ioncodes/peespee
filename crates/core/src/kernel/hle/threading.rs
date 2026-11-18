use crate::Memory;
use crate::cpu::{REG_A0, REG_A1, REG_SP};
use crate::kernel::Syscall;

const STACK_ALIGNMENT: u32 = 16;

impl<M: Memory> Syscall<'_, M> {
    pub fn create_thread(&mut self) {
        let handle = self.kernel.allocate_handle();
        let name = self.arg_as_str(0);
        self.kernel.threads.create_thread(
            handle,
            name,
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

        let thread = &mut self.kernel.threads[handle];
        let sp = (thread.context.registers[REG_SP as usize] - arg_size) & !(STACK_ALIGNMENT - 1);
        self.mmio.write(sp, &arguments);

        thread.context.registers[REG_A0 as usize] = arg_size;
        thread.context.registers[REG_A1 as usize] = sp;
        thread.context.registers[REG_SP as usize] = sp;
        thread.ready = true;
        tracing::info!(thread = %thread.name, "thread started");

        self.set_result(0);
    }

    pub fn exit_delete_thread(&mut self) {
        self.kernel
            .threads
            .exit_current(self.arg(0) as i32, self.cpu);
        self.set_result(0);
    }

    pub fn kernel_sleep_thread_callback(&mut self) {
        self.set_result(0);

        let handle = self.kernel.threads.current_thread;
        let thread = &mut self.kernel.threads[handle];
        thread.context.save(self.cpu);
        thread.ready = false;

        self.kernel.threads.schedule_next(self.cpu);
    }
}
