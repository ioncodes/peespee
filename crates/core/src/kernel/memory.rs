use std::collections::BTreeMap;

use crate::mmio::{MAIN_MEMORY_ADDR, addr::Address};

// TODO: this is mega ghetto and just a poc for now
// TODO: prob needs to start higher + reserve the loaded module

pub(crate) struct MemoryManager {
    blocks: BTreeMap<u32, Address>,
    next_address: Address,
}

impl MemoryManager {
    pub(crate) fn new() -> Self {
        Self {
            blocks: BTreeMap::new(),
            next_address: Address(MAIN_MEMORY_ADDR),
        }
    }

    pub(crate) fn alloc(&mut self, handle: u32, size: usize, name: &str) {
        let addr = self.next_address;
        tracing::info!(handle, %addr, size = format_args!("{size:08X}"), name, "allocating block");

        self.blocks.insert(handle, addr);
        self.next_address = addr.wrapping_add(size);
    }

    pub(crate) fn free(&mut self, handle: u32) {
        self.blocks.remove(&handle);
    }

    pub(crate) fn address_of(&self, handle: u32) -> Address {
        self.blocks[&handle]
    }
}
