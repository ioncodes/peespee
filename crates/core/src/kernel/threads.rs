use std::collections::BTreeMap;
use std::ops::{Index, IndexMut};

use crate::Cpu;
use crate::cpu::{REG_RA, REG_SP};

use super::MODULE_RETURN_STUB;

pub(super) const ROOT_THREAD: u32 = 0;
const THREAD_STACKS: u32 = 0x09F0_0000;

pub(crate) struct ThreadManager {
    threads: BTreeMap<u32, Thread>,
    pub(super) current_thread: u32,
    next_stack: u32,
}

impl ThreadManager {
    pub(crate) fn new() -> Self {
        let mut root_thread = Thread::new(ROOT_THREAD, "kernel".to_string(), 0, 0);
        root_thread.ready = true;

        Self {
            threads: BTreeMap::from([(ROOT_THREAD, root_thread)]),
            current_thread: ROOT_THREAD,
            next_stack: THREAD_STACKS,
        }
    }

    pub(crate) fn create_thread(
        &mut self,
        handle: u32,
        name: String,
        addr: u32,
        priority: u8,
        stack_size: u32,
    ) {
        let mut thread = Thread::new(handle, name, addr, priority);
        thread.context.registers[REG_SP as usize] = self.next_stack;
        thread.context.registers[REG_RA as usize] = MODULE_RETURN_STUB;
        self.next_stack -= stack_size;

        self.threads.insert(handle, thread);
    }

    pub(crate) fn exit_current(&mut self, status: i32, cpu: &mut Cpu) {
        let thread = self.threads.remove(&self.current_thread).unwrap();
        tracing::info!(status, thread = %thread.name, "thread exited");
        self.schedule_next(cpu);
    }

    pub(crate) fn schedule_next(&mut self, cpu: &mut Cpu) {
        let ready_threads = self.threads.values().filter(|t| t.ready);
        if let Some(next_thread) = ready_threads.min_by_key(|t| t.priority) {
            next_thread.context.restore(cpu);

            self.current_thread = next_thread.handle;
            tracing::info!(thread = %next_thread.name, "scheduled next thread");
        }
    }

    pub(crate) fn has_active_threads(&self) -> bool {
        self.threads.values().any(|t| t.ready)
    }
}

impl Index<u32> for ThreadManager {
    type Output = Thread;

    fn index(&self, handle: u32) -> &Thread {
        &self.threads[&handle]
    }
}

impl IndexMut<u32> for ThreadManager {
    fn index_mut(&mut self, handle: u32) -> &mut Thread {
        self.threads.get_mut(&handle).unwrap()
    }
}

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

    pub fn save(&mut self, cpu: &Cpu) {
        for (index, value) in self.registers.iter_mut().enumerate() {
            *value = cpu.register(index as u8);
        }
        self.pc = cpu.pc();
    }
}

pub struct Callback {
    pub owner: u32,
    pub entry: u32,
    pub common: u32,
}
