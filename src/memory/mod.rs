pub mod buffer;
pub mod exception;
pub mod mmap;

pub trait Bus {
    fn fetch(&self, paddr: u64) -> u32;

    fn read8(&self, paddr: u64) -> anyhow::Result<u8>;
    fn read16(&self, paddr: u64) -> anyhow::Result<u16>;
    fn read32(&self, paddr: u64) -> anyhow::Result<u32>;
    fn read64(&self, paddr: u64) -> anyhow::Result<u64>;

    fn write8(&mut self, paddr: u64, val: u8) -> anyhow::Result<()>;
    fn write16(&mut self, paddr: u64, val: u16) -> anyhow::Result<()>;
    fn write32(&mut self, paddr: u64, val: u32) -> anyhow::Result<()>;
    fn write64(&mut self, paddr: u64, val: u64) -> anyhow::Result<()>;
}
