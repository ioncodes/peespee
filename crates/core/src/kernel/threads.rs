use crate::Cpu;
use crate::cpu::{REG_RA, REG_SP};

use super::MODULE_RETURN_STUB;

const THREAD_STACKS: u32 = 0x09F0_0000;

pub(crate) struct ThreadManager {
    threads: Vec<Thread>,
    current_thread: Thread,

    next_handle: u32,
    next_stack: u32,
}

impl ThreadManager {
    pub(crate) fn new() -> Self {
        let root_thread = Thread::new(0, "kernel".to_string(), 0, 0);
        Self {
            threads: vec![root_thread.clone()],
            current_thread: root_thread,

            next_handle: 1,
            next_stack: THREAD_STACKS,
        }
    }

    pub(crate) fn create_thread(&mut self, name: String, addr: u32, priority: u8, stack_size: u32) -> u32 {
        let mut thread = Thread::new(self.next_handle, name, addr, priority);
        thread.context.registers[REG_SP as usize] = self.next_stack;
        thread.context.registers[REG_RA as usize] = MODULE_RETURN_STUB;
        self.next_stack -= stack_size;

        self.next_handle += 1;
        self.threads.push(thread);
        self.next_handle - 1
    }

    pub(crate) fn exit_current(&mut self, status: i32, cpu: &mut Cpu) {
        self.threads.retain(|t| t != &self.current_thread);
        tracing::info!(status, thread = %self.current_thread.name, "thread exited");
        self.schedule_next(cpu);
    }

    pub(crate) fn schedule_next(&mut self, cpu: &mut Cpu) {
        let ready_threads = self.threads.iter().filter(|t| t.ready);
        if let Some(next_thread) = ready_threads.min_by_key(|t| t.priority) {
            next_thread.context.restore(cpu);

            self.current_thread = next_thread.clone();
            tracing::info!(thread = %self.current_thread.name, "scheduled next thread");
        }
    }

    pub(crate) fn has_threads(&self) -> bool {
        !self.threads.is_empty()
    }

    pub(crate) fn find_thread(&self, handle: u32) -> &Thread {
        self.threads.iter().find(|t| t.handle == handle).unwrap()
    }

    pub(crate) fn find_thread_mut(&mut self, handle: u32) -> &mut Thread {
        self.threads.iter_mut().find(|t| t.handle == handle).unwrap()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Thread {
    pub handle: u32,
    pub name: String,
    pub context: ThreadContext,
    pub priority: u8,
    pub ready: bool,
}

impl Thread {
    pub fn new(handle: u32, name: String, addr: u32, priority: u8) -> Self {
        tracing::info!(name = %name, addr = format!("{:08x}", addr), priority, "thread created");
        Self {
            handle,
            name,
            priority,
            context: ThreadContext::new(addr),
            ready: false,
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct ThreadContext {
    pub registers: [u32; 32],
    pub pc: u32,
}

impl ThreadContext {
    pub fn new(pc: u32) -> Self {
        Self {
            registers: [0; 32],
            pc,
        }
    }

    pub fn restore(&self, cpu: &mut Cpu) {
        for (index, value) in self.registers.iter().enumerate() {
            cpu.set_register(index as u8, *value);
        }
        cpu.set_pc(self.pc);
    }
}
