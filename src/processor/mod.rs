use crate::memory::Bus;

pub mod riscv;

pub trait Cpu {
    fn step<B>(&mut self, bus: &mut B) -> anyhow::Result<()>
    where
        B: Bus;
}
