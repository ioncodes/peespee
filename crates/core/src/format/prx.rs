use goblin::{
    elf::{
        Elf,
        header::ET_EXEC,
        program_header::{PT_LOAD, ProgramHeader},
        reloc::{R_MIPS_26, R_MIPS_NONE},
        section_header::SHT_LOPROC,
    },
    elf32::reloc::r_type,
    error::Error,
};

use crate::{Memory, mmio::addr::Address};

const SHT_PSP_REL: u32 = SHT_LOPROC + 0xA0;
const ET_SCE_PRX: u16 = 0xffa0;
const REL_ENTRY_SIZE: usize = 8;

pub struct Prx<'a> {
    elf: Elf<'a>,
    buffer: &'a [u8],
}

impl<'a> Prx<'a> {
    pub fn new(buffer: &'a [u8]) -> Result<Self, Error> {
        let elf = Elf::parse(buffer)?;
        if !matches!(elf.header.e_type, ET_EXEC | ET_SCE_PRX) {
            return Err(Error::Malformed(format!(
                "unsupported ELF type: {:#06x}",
                elf.header.e_type
            )));
        }

        Ok(Self { elf, buffer })
    }

    pub fn entry(&self) -> u32 {
        self.elf.header.e_entry as u32
    }

    pub fn size(&self) -> usize {
        self.load_segments()
            .map(|ph| (ph.p_vaddr + ph.p_memsz) as usize)
            .max()
            .unwrap_or(0)
    }

    pub fn load_into<M: Memory>(&self, memory: &mut M, base_addr: Address) -> Result<Address, Error> {
        let relocatable = self.elf.header.e_type == ET_SCE_PRX;
        let base_addr = if relocatable { base_addr } else { Address(0) };
        let entry = base_addr + Address(self.entry());

        self.load_segments_into(memory, base_addr)?;
        if relocatable {
            self.apply_relocations(memory, base_addr)?;
        }

        Ok(entry)
    }

    fn load_segments(&self) -> impl Iterator<Item = &ProgramHeader> {
        self.elf
            .program_headers
            .iter()
            .filter(|ph| ph.p_type == PT_LOAD)
    }

    fn load_segments_into<M: Memory>(&self, memory: &mut M, base_addr: Address) -> Result<(), Error> {
        for ph in self.load_segments() {
            if ph.p_filesz > ph.p_memsz {
                return Err(Error::Malformed(
                    "segment file size exceeds memory size".into(),
                ));
            }

            let offset = ph.p_offset as usize;
            let data = &self.buffer[offset..offset + ph.p_filesz as usize];
            let vaddr = base_addr + Address(ph.p_vaddr as u32);
            memory.write(vaddr, data);
            memory.fill(
                vaddr.wrapping_add(data.len()),
                ph.p_memsz as usize - data.len(),
                0,
            );
        }

        Ok(())
    }

    fn apply_relocations<M: Memory>(&self, memory: &mut M, base_addr: Address) -> Result<(), Error> {
        for sh in self
            .elf
            .section_headers
            .iter()
            .filter(|sh| sh.sh_type == SHT_PSP_REL)
        {
            let offset = sh.sh_offset as usize;
            let data = &self.buffer[offset..offset + sh.sh_size as usize];
            let (entries, rest) = data.as_chunks::<REL_ENTRY_SIZE>();
            if !rest.is_empty() {
                return Err(Error::Malformed("incomplete relocation entry".into()));
            }

            for entry in entries {
                let r_offset = u32::from_le_bytes(entry[0..4].try_into().unwrap());
                let r_info = u32::from_le_bytes(entry[4..8].try_into().unwrap());

                let patch_segment = &self.elf.program_headers[((r_info >> 8) & 0xff) as usize];
                let target_segment = &self.elf.program_headers[((r_info >> 16) & 0xff) as usize];
                let offset =
                    base_addr + Address(patch_segment.p_vaddr as u32) + Address(r_offset);
                let target_base = u32::from(base_addr + Address(target_segment.p_vaddr as u32));

                match r_type(r_info) {
                    R_MIPS_NONE => {}
                    R_MIPS_26 => {
                        let mut bytes = [0; 4];
                        memory.read(offset, &mut bytes);
                        let insn = u32::from_le_bytes(bytes);
                        let target = (insn & 0x03ff_ffff).wrapping_add(target_base >> 2);
                        let patched = (insn & 0xfc00_0000) | (target & 0x03ff_ffff);
                        memory.write(offset, &patched.to_le_bytes());
                    }
                    x => {
                        return Err(Error::Malformed(format!(
                            "unsupported relocation type: {x}"
                        )));
                    }
                }
            }
        }

        Ok(())
    }
}
