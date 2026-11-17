use std::ffi::CStr;

use goblin::elf::{
    Elf,
    header::ET_EXEC,
    program_header::PT_LOAD,
    reloc::{R_MIPS_26, R_MIPS_NONE},
    section_header::SHT_LOPROC,
};
use goblin::error::Error;

use crate::{Memory, mmio::addr::Address};

const ET_SCE_PRX: u16 = 0xFFA0;
const SHT_PSP_REL: u32 = SHT_LOPROC + 0xA0;

const MODULE_INFO_SECTION: &str = ".rodata.sceModuleInfo";
const MODULE_INFO_OFFSET_MASK: u64 = 0x7FFF_FFFF;
const IMPORT_ENTRY_SIZE: usize = 0x14;
const RELOCATION_SIZE: usize = 8;
const STUB_SIZE: u32 = 8;
const JUMP_TARGET_MASK: u32 = 0x03FF_FFFF;

pub struct Prx {
    pub name: String,
    pub gp: u32,
    pub entry: u32,
    pub relocatable: bool,
    pub segments: Vec<Segment>,
    pub relocations: Vec<Relocation>,
    pub imports: Vec<Import>,
}

impl Prx {
    pub fn parse(buffer: &[u8]) -> Result<Self, Error> {
        let elf = Elf::parse(buffer)?;
        if !matches!(elf.header.e_type, ET_EXEC | ET_SCE_PRX) {
            return Err(Error::Malformed(format!(
                "unsupported ELF type: {:#06x}",
                elf.header.e_type
            )));
        }

        let half =
            |offset: usize| u16::from_le_bytes(buffer[offset..offset + 2].try_into().unwrap());
        let word =
            |offset: usize| u32::from_le_bytes(buffer[offset..offset + 4].try_into().unwrap());
        let string = |offset: usize| {
            CStr::from_bytes_until_nul(&buffer[offset..])
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned()
        };
        let load_segments: Vec<_> = elf
            .program_headers
            .iter()
            .filter(|ph| ph.p_type == PT_LOAD)
            .collect();
        let file_offset = |address: u32| {
            let address = address as u64;
            let segment = load_segments
                .iter()
                .find(|ph| (ph.p_vaddr..ph.p_vaddr + ph.p_filesz).contains(&address))
                .unwrap_or_else(|| panic!("address {address:#010x} is not backed by the file"));
            (address - segment.p_vaddr + segment.p_offset) as usize
        };

        let relocatable = elf.header.e_type == ET_SCE_PRX;
        let module_info = if relocatable {
            (elf.program_headers[0].p_paddr & MODULE_INFO_OFFSET_MASK) as usize
        } else {
            elf.section_headers
                .iter()
                .find(|sh| elf.shdr_strtab.get_at(sh.sh_name) == Some(MODULE_INFO_SECTION))
                .expect("executable has no module info section")
                .sh_offset as usize
        };

        let mut imports = Vec::new();
        for address in
            (word(module_info + 0x2C)..word(module_info + 0x30)).step_by(IMPORT_ENTRY_SIZE)
        {
            let entry = file_offset(address);
            let size = buffer[entry + 0x08] as usize * 4;
            assert_eq!(
                size, IMPORT_ENTRY_SIZE,
                "unsupported import entry size {size} at {address:#010x}"
            );

            let nid_table = file_offset(word(entry + 0x0C));
            let stub_table = word(entry + 0x10);
            imports.push(Import {
                library: string(file_offset(word(entry))),
                functions: (0..half(entry + 0x0A) as usize)
                    .map(|n| Function {
                        nid: word(nid_table + n * 4),
                        stub: stub_table + n as u32 * STUB_SIZE,
                    })
                    .collect(),
            });
        }

        let segments = load_segments
            .iter()
            .map(|ph| {
                assert!(
                    ph.p_filesz <= ph.p_memsz,
                    "segment file size exceeds memory size"
                );

                let offset = ph.p_offset as usize;
                Segment {
                    address: ph.p_vaddr as u32,
                    data: buffer[offset..offset + ph.p_filesz as usize].to_vec(),
                    memory_size: ph.p_memsz as usize,
                }
            })
            .collect();

        let mut relocations = Vec::new();
        for sh in elf
            .section_headers
            .iter()
            .filter(|sh| sh.sh_type == SHT_PSP_REL)
        {
            assert!(
                sh.sh_size as usize % RELOCATION_SIZE == 0,
                "incomplete relocation entry"
            );

            let start = sh.sh_offset as usize;
            for entry in (start..start + sh.sh_size as usize).step_by(RELOCATION_SIZE) {
                let patch_segment = &elf.program_headers[buffer[entry + 5] as usize];
                let target_segment = &elf.program_headers[buffer[entry + 6] as usize];
                relocations.push(Relocation {
                    kind: buffer[entry + 4],
                    address: patch_segment.p_vaddr as u32 + word(entry),
                    target_base: target_segment.p_vaddr as u32,
                });
            }
        }

        Ok(Self {
            name: string(module_info + 0x04),
            gp: word(module_info + 0x20),
            entry: elf.header.e_entry as u32,
            relocatable,
            segments,
            relocations,
            imports,
        })
    }

    pub fn load_base(&self, requested: impl Into<Address>) -> Address {
        if self.relocatable {
            requested.into()
        } else {
            Address(0)
        }
    }

    pub fn load_into(
        &self,
        memory: &mut impl Memory,
        requested_base: impl Into<Address>,
    ) -> Address {
        let base = self.load_base(requested_base);
        for segment in &self.segments {
            let address = base + Address(segment.address);
            memory.write(address, &segment.data);
            memory.fill(
                address.wrapping_add(segment.data.len()),
                segment.memory_size - segment.data.len(),
                0,
            );
        }

        if self.relocatable {
            for relocation in &self.relocations {
                let address = base + Address(relocation.address);
                let target_base = u32::from(base + Address(relocation.target_base));
                match relocation.kind as u32 {
                    R_MIPS_NONE => {}
                    R_MIPS_26 => {
                        let instruction = memory.read32(address);
                        let target =
                            (instruction & JUMP_TARGET_MASK).wrapping_add(target_base >> 2);
                        memory.write32(
                            address,
                            (instruction & !JUMP_TARGET_MASK) | (target & JUMP_TARGET_MASK),
                        );
                    }
                    kind => unimplemented!("relocation type {kind} at {address}"),
                }
            }
        }

        base + Address(self.entry)
    }
}

pub struct Segment {
    pub address: u32,
    pub data: Vec<u8>,
    pub memory_size: usize,
}

pub struct Relocation {
    pub kind: u8,
    pub address: u32,
    pub target_base: u32,
}

pub struct Import {
    pub library: String,
    pub functions: Vec<Function>,
}

pub struct Function {
    pub nid: u32,
    pub stub: u32,
}
