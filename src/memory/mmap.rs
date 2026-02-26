use crate::memory::{Bus, buffer::MemoryBuffer, exception::Trap};

#[derive(Debug)]
pub struct Mmap {
    ram: MemoryBuffer,
    ram_start: u64,
}

impl Mmap {
    pub fn new(origin: u64, length: usize) -> Self {
        Self {
            ram: MemoryBuffer::new(length),
            ram_start: origin,
        }
    }

    pub fn load_segment(&mut self, src: &[u8], paddr: u64, memsz: u64, filesz: u64) {
        unsafe {
            let dst = self.ram.as_mut_ptr().add((paddr - self.ram_start) as usize);
            std::ptr::copy_nonoverlapping(src.as_ptr(), dst, filesz as usize);

            if memsz > filesz {
                std::ptr::write_bytes(dst.add(filesz as usize), 0, (memsz - filesz) as usize);
            }
        }
    }

    #[inline(always)]
    fn load<T>(&self, paddr: u64) -> anyhow::Result<T>
    where
        T: Copy,
    {
        if !paddr.is_multiple_of(std::mem::size_of::<T>() as u64) {
            return Err(Trap::MisalignedLoad {
                addr: paddr,
                align: std::mem::size_of::<T>(),
            }
            .into());
        }
        if paddr >= self.ram_start {
            Ok(self.ram.load(paddr - self.ram_start))
        } else {
            Err(Trap::LoadAccessFault { addr: paddr }.into())
        }
    }

    #[inline(always)]
    fn store<T>(&mut self, paddr: u64, val: T) -> anyhow::Result<()> {
        if !paddr.is_multiple_of(std::mem::size_of::<T>() as u64) {
            return Err(Trap::MisalignedStore {
                addr: paddr,
                align: std::mem::size_of::<T>(),
            }
            .into());
        }
        if paddr >= self.ram_start {
            self.ram.store(paddr - self.ram_start, val);
        } else {
            return Err(Trap::StoreAccessFault { addr: paddr }.into());
        }
        Ok(())
    }
}

impl Bus for Mmap {
    #[inline(always)]
    fn fetch(&self, paddr: u64) -> u32 {
        self.ram.load::<u32>(paddr - self.ram_start)
    }

    fn read8(&self, paddr: u64) -> anyhow::Result<u8> {
        self.load(paddr)
    }

    fn read16(&self, paddr: u64) -> anyhow::Result<u16> {
        self.load(paddr)
    }

    fn read32(&self, paddr: u64) -> anyhow::Result<u32> {
        self.load(paddr)
    }

    fn read64(&self, paddr: u64) -> anyhow::Result<u64> {
        self.load(paddr)
    }

    fn write8(&mut self, paddr: u64, val: u8) -> anyhow::Result<()> {
        self.store(paddr, val)
    }

    fn write16(&mut self, paddr: u64, val: u16) -> anyhow::Result<()> {
        self.store(paddr, val)
    }

    fn write32(&mut self, paddr: u64, val: u32) -> anyhow::Result<()> {
        self.store(paddr, val)
    }

    fn write64(&mut self, paddr: u64, val: u64) -> anyhow::Result<()> {
        self.store(paddr, val)
    }
}
