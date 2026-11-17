use crate::format::Prx;
use crate::mmio::addr::Address;
use crate::{Cpu, Memory};

use super::nids::{self, *};
use super::{IMPORT_SYSCALL_BASE, Kernel, MODULE_RETURN_SYSCALL, SYSCALL};

const JR_RA: u32 = 0x03E0_0008;

impl Kernel {
    pub(crate) fn dispatch(&mut self, syscall: u32, cpu: &mut Cpu, mmio: &mut impl Memory) {
        if syscall == MODULE_RETURN_SYSCALL {
            self.return_from_module(cpu, mmio);
            return;
        }

        match self.imports[(syscall - IMPORT_SYSCALL_BASE) as usize] {
            SCE_KERNEL_CREATE_THREAD => todo!("implement sceKernelCreateThread"),
            nid => {
                let name = nids::name(nid).unwrap_or("???");
                todo!("Unimplemented import {nid:08X} [{name}]")
            }
        }
    }

    pub(crate) fn link_imports(&mut self, prx: &Prx, base: Address, memory: &mut impl Memory) {
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

    fn return_from_module(&mut self, cpu: &mut Cpu, _mmio: &mut impl Memory) {
        // The module entry point returns its status in $v0.
        self.threads.exit_current(cpu.register(2) as i32);
    }
}
