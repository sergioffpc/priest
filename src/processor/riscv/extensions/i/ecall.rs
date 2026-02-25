use crate::{
    memory::Bus,
    processor::riscv::{hart::Hart, instruction::InstrExec},
};

#[derive(Debug)]
pub struct Ebreak;

impl InstrExec for Ebreak {
    #[inline(always)]
    fn matches(&self, inst: u32) -> bool {
        inst == 0x73
    }

    #[inline(always)]
    #[cfg_attr(feature = "trace", tracing::instrument(name = "ECALL", skip_all))]
    fn call(&self, _inst: u32, hart: &mut Hart, _bus: &mut dyn Bus) -> anyhow::Result<()> {
        hart.next_pc();
        Ok(())
    }
}
