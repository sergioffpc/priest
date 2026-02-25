use crate::{
    memory::Bus,
    processor::riscv::{hart::Hart, instruction::InstrExec},
};

#[derive(Debug)]
pub struct Fence;

impl InstrExec for Fence {
    #[inline(always)]
    fn matches(&self, inst: u32) -> bool {
        inst & 0x707f == 0xf
    }

    #[inline(always)]
    #[cfg_attr(feature = "trace", tracing::instrument(name = "FENCE", skip_all))]
    fn call(&self, inst: u32, hart: &mut Hart, _bus: &mut dyn Bus) -> anyhow::Result<()> {
        let funct3 = (inst >> 12) & 0x7;
        match funct3 {
            0 => {}
            1 => {}
            _ => {}
        }

        hart.next_pc();
        Ok(())
    }
}
