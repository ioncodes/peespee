pub mod decoder;
mod interpreter;

use crate::cpu::{decoder::Instruction, interpreter::Interpreter};

pub const REG_A0: u8 = 4;
pub const REG_A1: u8 = 5;
pub const REG_GP: u8 = 28;
pub const REG_SP: u8 = 29;
pub const REG_RA: u8 = 31;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[must_use]
pub enum Outcome {
    Continue,
    Syscall(u32),
}

pub struct Cpu {
    gprs: [u32; 32],
    hi: u32,
    lo: u32,
    pc: u32,
    ll_bit: bool, // for LL and SC
    pub(crate) next_pc: u32,
}

impl Cpu {
    pub fn new(pc: u32) -> Self {
        Self {
            gprs: [0; 32],
            pc,
            next_pc: pc.wrapping_add(4),
            hi: 0,
            lo: 0,
            ll_bit: false,
        }
    }

    #[inline(always)]
    pub fn step(&mut self, memory: &mut impl crate::Memory) -> Outcome {
        let instr_pc = self.pc;
        let instr = memory.read32(instr_pc);
        //println!("[{:08x}] {}", self.pc, Instruction(instr));

        self.pc = self.next_pc;
        self.next_pc = self.next_pc.wrapping_add(4);

        let mut interpreter = Interpreter {
            cpu: self,
            mmio: memory,
            instr_pc,
            outcome: Outcome::Continue,
        };
        decoder::run_ops(&mut interpreter, instr);
        interpreter.outcome
    }

    pub fn pc(&self) -> u32 {
        self.pc
    }

    pub fn set_pc(&mut self, pc: u32) {
        self.pc = pc;
        self.next_pc = pc.wrapping_add(4);
    }

    pub fn register(&self, index: u8) -> u32 {
        self.gprs[index as usize]
    }

    pub fn set_register(&mut self, index: u8, value: u32) {
        if index != 0 {
            self.gprs[index as usize] = value;
        }
    }
}
