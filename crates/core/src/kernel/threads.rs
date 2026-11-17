const SCE_KERNEL_USER_HIGHEST_PRIORITY: u8 = 16;
const SCE_KERNEL_USER_LOWEST_PRIORITY: u8 = 111;

pub(crate) struct ThreadManager {
    threads: Vec<Thread>,
    current_thread: Thread,
}

impl ThreadManager {
    pub(crate) fn new() -> Self {
        let root_thread = Thread::new("kernel".to_string(), 0);
        Self {
            threads: vec![root_thread.clone()],
            current_thread: root_thread,
        }
    }

    pub(crate) fn exit_current(&mut self, status: i32) {
        self.threads.retain(|t| t != &self.current_thread);
        tracing::info!(status, thread = %self.current_thread.name, "thread exited");
        self.schedule_next();
    }

    pub(crate) fn schedule_next(&mut self) {
        if let Some(next_thread) = self.threads.iter().min_by_key(|t| t.priority) {
            self.current_thread = next_thread.clone();
            tracing::info!(thread = %self.current_thread.name, "scheduled next thread");
        }
    }

    pub(crate) fn has_threads(&self) -> bool {
        !self.threads.is_empty()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Thread {
    pub name: String,
    pub priority: u8,
}

impl Thread {
    pub fn new(name: String, priority: u8) -> Self {
        tracing::info!(name = %name, priority, "thread created");
        Self { name, priority }
    }
}
