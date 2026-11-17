pub mod addr;

use std::collections::HashMap;

use addr::Address;

const SCRATCHPAD_SIZE: usize = 0x4000;
const VIDEO_MEMORY_SIZE: usize = 0x20_0000;
const MAIN_MEMORY_SIZE: usize = 0x200_0000;
const EXCEPTION_VECTORS_SIZE: usize = 0x10_0000;

pub struct Mmio {
    scratchpad: Vec<u8>,
    video_memory: Vec<u8>,
    main_memory: Vec<u8>,
    exception_vectors: Vec<u8>,
    io_readback: HashMap<u32, u8>,
}

impl Mmio {
    pub fn new() -> Self {
        Self {
            scratchpad: vec![0; SCRATCHPAD_SIZE],
            video_memory: vec![0; VIDEO_MEMORY_SIZE],
            main_memory: vec![0; MAIN_MEMORY_SIZE],
            exception_vectors: vec![0; EXCEPTION_VECTORS_SIZE],
            io_readback: HashMap::new(),
        }
    }
}

impl Memory for Mmio {
    fn read8(&self, addr: impl Into<Address>) -> u8 {
        let addr = u32::from(addr.into()) & 0x3fff_ffff;
        match addr {
            // Normal memory stuff
            0x0001_0000..=0x0001_3fff => self.scratchpad[(addr - 0x0001_0000) as usize],
            0x0400_0000..=0x041f_ffff => self.video_memory[(addr - 0x0400_0000) as usize],
            0x0800_0000..=0x09ff_ffff => self.main_memory[(addr - 0x0800_0000) as usize],
            0x1fc0_0000..=0x1fcf_ffff => self.exception_vectors[(addr - 0x1fc0_0000) as usize],
            // MMIO
            0x1c00_0000..=0x1fbf_ffff | 0x1fd0_0000..=0x1fd0_ffff => {
                tracing::error!("MMIO read8 at address {:08x}", addr);
                self.io_readback.get(&addr).copied().unwrap_or(0)
            }
            _ => unreachable!("Invalid read8 address {:08x}", addr),
        }
    }

    fn write8(&mut self, addr: impl Into<Address>, value: u8) {
        let addr = u32::from(addr.into()) & 0x3fff_ffff;
        match addr {
            // Normal memory stuff
            0x0001_0000..=0x0001_3fff => self.scratchpad[(addr - 0x0001_0000) as usize] = value,
            0x0400_0000..=0x041f_ffff => self.video_memory[(addr - 0x0400_0000) as usize] = value,
            0x0800_0000..=0x09ff_ffff => self.main_memory[(addr - 0x0800_0000) as usize] = value,
            0x1fc0_0000..=0x1fcf_ffff => {
                self.exception_vectors[(addr - 0x1fc0_0000) as usize] = value
            }
            // MMIO
            0x1c00_0000..=0x1fbf_ffff | 0x1fd0_0000..=0x1fd0_ffff => {
                tracing::error!("MMIO write8 at address {:08x} = {:02x}", addr, value);
                self.io_readback.insert(addr, value);
            }
            _ => unreachable!("Invalid write8 address {:08x}", addr),
        }
    }
}

pub trait Memory {
    fn read8(&self, addr: impl Into<Address>) -> u8;
    fn write8(&mut self, addr: impl Into<Address>, value: u8);

    fn read16(&self, addr: impl Into<Address>) -> u16 {
        let addr = addr.into();
        let low = self.read8(addr) as u16;
        let high = self.read8(addr.wrapping_add(1)) as u16;
        (high << 8) | low
    }

    fn write16(&mut self, addr: impl Into<Address>, value: u16) {
        let addr = addr.into();
        let low = (value & 0xff) as u8;
        let high = (value >> 8) as u8;
        self.write8(addr, low);
        self.write8(addr.wrapping_add(1), high);
    }

    fn read32(&self, addr: impl Into<Address>) -> u32 {
        let addr = addr.into();
        let b0 = self.read8(addr) as u32;
        let b1 = self.read8(addr.wrapping_add(1)) as u32;
        let b2 = self.read8(addr.wrapping_add(2)) as u32;
        let b3 = self.read8(addr.wrapping_add(3)) as u32;
        (b3 << 24) | (b2 << 16) | (b1 << 8) | b0
    }

    fn write32(&mut self, addr: impl Into<Address>, value: u32) {
        let addr = addr.into();
        let b0 = (value & 0xff) as u8;
        let b1 = ((value >> 8) & 0xff) as u8;
        let b2 = ((value >> 16) & 0xff) as u8;
        let b3 = ((value >> 24) & 0xff) as u8;
        self.write8(addr, b0);
        self.write8(addr.wrapping_add(1), b1);
        self.write8(addr.wrapping_add(2), b2);
        self.write8(addr.wrapping_add(3), b3);
    }

    fn read(&self, addr: impl Into<Address>, buffer: &mut [u8]) {
        let addr = addr.into();
        for (offset, byte) in buffer.iter_mut().enumerate() {
            *byte = self.read8(addr.wrapping_add(offset));
        }
    }

    fn write(&mut self, addr: impl Into<Address>, buffer: &[u8]) {
        let addr = addr.into();
        for (offset, byte) in buffer.iter().copied().enumerate() {
            self.write8(addr.wrapping_add(offset), byte);
        }
    }

    fn fill(&mut self, addr: impl Into<Address>, size: usize, value: u8) {
        let addr = addr.into();
        for offset in 0..size {
            self.write8(addr.wrapping_add(offset), value);
        }
    }
}
