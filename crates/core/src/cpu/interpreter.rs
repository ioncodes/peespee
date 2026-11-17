use crate::cpu::REG_RA;

use super::decoder::{
    ArithKind, BitfieldKind, BranchKind, FpuKind, Instruction, LoadStoreKind, LogicKind,
    MuldivKind, Ops, PspDebugKind, ShiftKind, SystemKind, VfpuAluKind, VfpuMiscKind,
};

pub(super) struct Interpreter<'a, M: crate::Memory> {
    pub(super) cpu: &'a mut super::Cpu,
    pub(super) mmio: &'a mut M,
    pub(super) instr_pc: u32,
    pub(super) outcome: super::Outcome,
}

impl<M: crate::Memory> Interpreter<'_, M> {
    fn overflow_exception(&mut self, instr: Instruction) {
        self.cpu.ll_bit = false;
        tracing::warn!(
            "integer overflow exception shit for {instr} at {:08x}",
            self.instr_pc
        );
    }

    fn jump(&mut self, target: u32) {
        self.cpu.next_pc = target;
    }
}

impl<M: crate::Memory> Ops for Interpreter<'_, M> {
    fn shift(&mut self, op: ShiftKind, instr: Instruction) {
        let rt = self.cpu.register(instr.rt());
        let rs = self.cpu.register(instr.rs());

        match op {
            ShiftKind::Sll => self.cpu.set_register(instr.rd(), rt << instr.sa()),
            ShiftKind::Srl => self.cpu.set_register(instr.rd(), rt >> instr.sa()),
            ShiftKind::Sllv => self.cpu.set_register(instr.rd(), rt << (rs & 0x1F)),
            ShiftKind::Srlv => self.cpu.set_register(instr.rd(), rt >> (rs & 0x1F)),
            ShiftKind::Rotr => self
                .cpu
                .set_register(instr.rd(), rt.rotate_right(instr.sa() as u32)),
            ShiftKind::Sra => self
                .cpu
                .set_register(instr.rd(), ((rt as i32) >> instr.sa()) as u32),
            ShiftKind::Rotrv => self
                .cpu
                .set_register(instr.rd(), rt.rotate_right(rs & 0x1F)),
            ShiftKind::Srav => self
                .cpu
                .set_register(instr.rd(), ((rt as i32) >> (rs & 0x1F)) as u32),
        }
    }

    fn arith(&mut self, op: ArithKind, instr: Instruction) {
        let rs = self.cpu.register(instr.rs());
        let rt = self.cpu.register(instr.rt());

        match op {
            ArithKind::Add => {
                let (value, overflow) = (rs as i32).overflowing_add(rt as i32);
                if overflow {
                    self.overflow_exception(instr);
                }
                self.cpu.set_register(instr.rd(), value as u32)
            }
            ArithKind::Addi => {
                let (value, overflow) = (rs as i32).overflowing_add(instr.imm16());
                if overflow {
                    self.overflow_exception(instr);
                }
                self.cpu.set_register(instr.rt(), value as u32)
            }
            ArithKind::Addu => self.cpu.set_register(instr.rd(), rs.wrapping_add(rt)),
            ArithKind::Addiu => self
                .cpu
                .set_register(instr.rt(), rs.wrapping_add(instr.imm16() as u32)),
            ArithKind::Sub => {
                let (value, overflow) = (rs as i32).overflowing_sub(rt as i32);
                if overflow {
                    self.overflow_exception(instr);
                }
                self.cpu.set_register(instr.rd(), value as u32)
            }
            ArithKind::Subu => self.cpu.set_register(instr.rd(), rs.wrapping_sub(rt)),
            ArithKind::Slt => self
                .cpu
                .set_register(instr.rd(), u32::from((rs as i32) < (rt as i32))),
            ArithKind::Slti => self
                .cpu
                .set_register(instr.rt(), u32::from((rs as i32) < instr.imm16())),
            ArithKind::Sltu => self.cpu.set_register(instr.rd(), u32::from(rs < rt)),
            ArithKind::Sltiu => self
                .cpu
                .set_register(instr.rt(), u32::from(rs < instr.imm16() as u32)),
            ArithKind::Max => self
                .cpu
                .set_register(instr.rd(), (rs as i32).max(rt as i32) as u32),
            ArithKind::Min => self
                .cpu
                .set_register(instr.rd(), (rs as i32).min(rt as i32) as u32),
            ArithKind::Movz if rt == 0 => self.cpu.set_register(instr.rd(), rs),
            ArithKind::Movn if rt != 0 => self.cpu.set_register(instr.rd(), rs),
            ArithKind::Movz | ArithKind::Movn => {}
            ArithKind::Clz => self.cpu.set_register(instr.rd(), rs.leading_zeros()),
            ArithKind::Clo => self.cpu.set_register(instr.rd(), rs.leading_ones()),
        }
    }

    fn logic(&mut self, op: LogicKind, instr: Instruction) {
        let rs = self.cpu.register(instr.rs());
        let rt = self.cpu.register(instr.rt());
        let uimm = instr.uimm() as u32;

        match op {
            LogicKind::Lui => self.cpu.set_register(instr.rt(), instr.upper()),
            LogicKind::And => self.cpu.set_register(instr.rd(), rs & rt),
            LogicKind::Or => self.cpu.set_register(instr.rd(), rs | rt),
            LogicKind::Xor => self.cpu.set_register(instr.rd(), rs ^ rt),
            LogicKind::Nor => self.cpu.set_register(instr.rd(), !(rs | rt)),
            LogicKind::Andi => self.cpu.set_register(instr.rt(), rs & uimm),
            LogicKind::Ori => self.cpu.set_register(instr.rt(), rs | uimm),
            LogicKind::Xori => self.cpu.set_register(instr.rt(), rs ^ uimm),
        }
    }

    fn muldiv(&mut self, op: MuldivKind, instr: Instruction) {
        todo!("{op:?} ({:08x})", instr.0)
    }

    fn branch(&mut self, op: BranchKind, instr: Instruction) {
        let rs = self.cpu.register(instr.rs());
        let rt = self.cpu.register(instr.rt());

        if matches!(
            op,
            BranchKind::Bltzal | BranchKind::Bgezal | BranchKind::Bltzall | BranchKind::Bgezall
        ) {
            self.cpu.set_register(REG_RA, self.cpu.pc.wrapping_add(4));
        }

        match op {
            BranchKind::Jr => self.jump(self.cpu.register(instr.rs())),
            BranchKind::Jalr => {
                let target = self.cpu.register(instr.rs());
                self.cpu
                    .set_register(instr.rd(), self.cpu.pc.wrapping_add(4));
                self.jump(target);
            }
            BranchKind::J => self.jump((self.cpu.pc & 0xF0000000) | instr.target()),
            BranchKind::Jal => {
                self.cpu.set_register(REG_RA, self.cpu.pc.wrapping_add(4));
                self.jump((self.cpu.pc & 0xF0000000) | instr.target());
            }
            BranchKind::Beq | BranchKind::Beql if rs == rt => {
                self.jump(self.cpu.pc.wrapping_add_signed(instr.offset()));
            }
            BranchKind::Bne | BranchKind::Bnel if rs != rt => {
                self.jump(self.cpu.pc.wrapping_add_signed(instr.offset()));
            }
            BranchKind::Blez | BranchKind::Blezl if (rs as i32) <= 0 => {
                self.jump(self.cpu.pc.wrapping_add_signed(instr.offset()));
            }
            BranchKind::Bgtz | BranchKind::Bgtzl if (rs as i32) > 0 => {
                self.jump(self.cpu.pc.wrapping_add_signed(instr.offset()));
            }
            BranchKind::Bltz | BranchKind::Bltzl if (rs as i32) < 0 => {
                self.jump(self.cpu.pc.wrapping_add_signed(instr.offset()));
            }
            BranchKind::Bgez | BranchKind::Bgezl if (rs as i32) >= 0 => {
                self.jump(self.cpu.pc.wrapping_add_signed(instr.offset()));
            }
            BranchKind::Bltzal | BranchKind::Bltzall if (rs as i32) < 0 => {
                self.jump(self.cpu.pc.wrapping_add_signed(instr.offset()));
            }
            BranchKind::Bgezal | BranchKind::Bgezall if (rs as i32) >= 0 => {
                self.jump(self.cpu.pc.wrapping_add_signed(instr.offset()));
            }
            BranchKind::Beql
            | BranchKind::Bnel
            | BranchKind::Blezl
            | BranchKind::Bgtzl
            | BranchKind::Bltzl
            | BranchKind::Bgezl
            | BranchKind::Bltzall
            | BranchKind::Bgezall => {
                self.cpu.pc = self.cpu.next_pc;
                self.cpu.next_pc = self.cpu.next_pc.wrapping_add(4);
            }
            _ => {}
        }
    }

    fn system(&mut self, op: SystemKind, instr: Instruction) {
        match op {
            SystemKind::Syscall => self.outcome = super::Outcome::Syscall(instr.code()),
            _ => todo!("{op:?} ({:08x})", instr.0),
        }
    }

    fn load_store(&mut self, op: LoadStoreKind, instr: Instruction) {
        let va = self
            .cpu
            .register(instr.rs())
            .wrapping_add_signed(instr.disp());
        let rt = self.cpu.register(instr.rt());

        match op {
            LoadStoreKind::Sb => self.mmio.write8(va, self.cpu.register(instr.rt()) as u8),
            LoadStoreKind::Sh => self.mmio.write16(va, self.cpu.register(instr.rt()) as u16),
            LoadStoreKind::Sw => self.mmio.write32(va, self.cpu.register(instr.rt())),
            LoadStoreKind::Lb => self
                .cpu
                .set_register(instr.rt(), self.mmio.read8(va) as i8 as i32 as u32),
            LoadStoreKind::Lh => self
                .cpu
                .set_register(instr.rt(), self.mmio.read16(va) as i16 as i32 as u32),
            LoadStoreKind::Lw => self
                .cpu
                .set_register(instr.rt(), self.mmio.read32(va) as u32),
            LoadStoreKind::Lwl => {
                let count = (va & 3) as usize + 1;
                let mut bytes = rt.to_le_bytes();
                self.mmio.read(va & !3, &mut bytes[4 - count..]);
                self.cpu.set_register(instr.rt(), u32::from_le_bytes(bytes));
            }
            LoadStoreKind::Lbu => self
                .cpu
                .set_register(instr.rt(), u32::from(self.mmio.read8(va))),
            LoadStoreKind::Lhu => self
                .cpu
                .set_register(instr.rt(), u32::from(self.mmio.read16(va))),
            LoadStoreKind::Lwr => {
                let count = 4 - (va & 3) as usize;
                let mut bytes = rt.to_le_bytes();
                self.mmio.read(va, &mut bytes[..count]);
                self.cpu.set_register(instr.rt(), u32::from_le_bytes(bytes));
            }
            LoadStoreKind::Swl => {
                let count = (va & 3) as usize + 1;
                self.mmio.write(va & !3, &rt.to_le_bytes()[4 - count..]);
            }
            LoadStoreKind::Swr => {
                let count = 4 - (va & 3) as usize;
                self.mmio.write(va, &rt.to_le_bytes()[..count]);
            }
            LoadStoreKind::Ll => {
                let value = self.mmio.read32(va);
                self.cpu.set_register(instr.rt(), value);
                self.cpu.ll_bit = true;
            }
            LoadStoreKind::Sc => {
                if self.cpu.ll_bit {
                    self.mmio.write32(va, rt);
                }
                self.cpu.set_register(instr.rt(), self.cpu.ll_bit as u32);
            }
        }
    }

    fn bitfield(&mut self, op: BitfieldKind, instr: Instruction) {
        todo!("{op:?} ({:08x})", instr.0)
    }

    fn fpu(&mut self, op: FpuKind, instr: Instruction) {
        todo!("{op:?} ({:08x})", instr.0)
    }

    fn psp_debug(&mut self, op: PspDebugKind, instr: Instruction) {
        todo!("{op:?} ({:08x})", instr.0)
    }

    fn vfpu_alu(&mut self, op: VfpuAluKind, instr: Instruction) {
        todo!("{op:?} ({:08x})", instr.0)
    }

    fn vfpu_misc(&mut self, op: VfpuMiscKind, instr: Instruction) {
        todo!("{op:?} ({:08x})", instr.0)
    }

    fn on_invalid(&mut self, instr: Instruction) {
        panic!("invalid instruction {:08x}", instr.0)
    }
}
