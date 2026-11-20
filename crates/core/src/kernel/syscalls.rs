use std::ffi::CStr;

use crate::format::Prx;
use crate::mmio::addr::Address;
use crate::{Cpu, Display, Memory};

use super::nids::{self, *};
use super::{IMPORT_SYSCALL_BASE, Kernel, MODULE_RETURN_SYSCALL, SYSCALL};

const JR_RA: u32 = 0x03E0_0008;

pub(crate) struct Syscall<'a, M: Memory> {
    pub kernel: &'a mut Kernel,
    pub cpu: &'a mut Cpu,
    pub mmio: &'a mut M,
    pub display: &'a mut Display,
}

impl<M: Memory> Syscall<'_, M> {
    pub fn arg(&self, index: u8) -> u32 {
        self.cpu.register(4 + index)
    }

    pub fn arg_as_str(&self, index: u8) -> String {
        let mut buffer = [0u8; 64]; // TODO: what's a universally good size? should we just read until null?
        self.mmio.read(self.arg(index), &mut buffer);
        CStr::from_bytes_until_nul(&buffer)
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned()
    }

    pub fn set_result(&mut self, value: u32) {
        self.cpu.set_register(2, value);
    }

    pub(crate) fn dispatch(&mut self, code: u32) {
        if code == MODULE_RETURN_SYSCALL {
            self.return_from_module();
            return;
        }

        let nid = self.kernel.imports[(code - IMPORT_SYSCALL_BASE) as usize];
        match nid {
            // threading
            SCE_KERNEL_CREATE_THREAD => self.create_thread(),
            SCE_KERNEL_START_THREAD => self.start_thread(),
            SCE_KERNEL_EXIT_DELETE_THREAD => self.exit_delete_thread(),
            SCE_KERNEL_SLEEP_THREAD_CB => self.kernel_sleep_thread_callback(),
            SCE_KERNEL_GET_THREAD_ID => self.get_thread_id(),
            SCE_KERNEL_REFER_THREAD_STATUS => self.refer_thread_status(),

            // callbacks
            SCE_KERNEL_CREATE_CALLBACK => self.create_callback(),
            SCE_KERNEL_REGISTER_EXIT_CALLBACK => self.register_exit_callback(),

            // alloc
            SCE_KERNEL_ALLOC_PARTITION_MEMORY => self.alloc_partition_memory(),
            SCE_KERNEL_GET_BLOCK_HEAD_ADDR => self.get_block_address(),

            // mutex
            SCE_KERNEL_CREATE_LW_MUTEX => self.create_user_mutex(),
            SCE_KERNEL_CREATE_SEMA => self.create_semaphore(),

            // stdio
            SCE_KERNEL_STDIN => self.stdin(),
            SCE_KERNEL_STDOUT => self.stdout(),
            SCE_KERNEL_STDERR => self.stderr(),

            // device io
            SCE_IO_DEVCTL => self.device_control(),

            // general io
            SCE_IO_WRITE => self.write_io(),

            // ge
            SCE_GE_EDRAM_GET_ADDR => self.get_edram_addr(),

            // display
            SCE_DISPLAY_SET_MODE => self.set_display_mode(),
            SCE_DISPLAY_SET_FRAME_BUF => self.set_frame_buffer(),

            // util
            SCE_UTILITY_GET_SYSTEM_PARAM_INT => self.get_system_parameter(),

            nid => {
                let name = nids::name(nid).unwrap_or("???");
                tracing::error!("Unimplemented import {nid:08X} [{name}]");
                self.set_result(0);
            }
        }
    }

    fn return_from_module(&mut self) {
        // The module entry point returns its status in $v0.
        self.kernel
            .threads
            .exit_current(self.cpu.register(2) as i32, self.cpu);
    }
}

impl Kernel {
    pub(crate) fn link_imports(
        &mut self,
        prx: &Prx,
        base: impl Into<Address>,
        memory: &mut impl Memory,
    ) {
        let base = base.into();
        for import in &prx.imports {
            for function in &import.functions {
                let code = IMPORT_SYSCALL_BASE + self.imports.len() as u32;
                self.imports.push(function.nid);

                let stub = base + Address(function.stub);
                memory.write32(stub, JR_RA);
                memory.write32(stub.wrapping_add(4), (code << 6) | SYSCALL);
            }
        }
    }
}
