use crate::{
    memory::Bus,
    processor::riscv::{hart::Hart, instruction::InstrExec},
};

#[derive(Debug)]
pub struct Srai;

impl InstrExec for Srai {
    #[inline(always)]
    fn matches(&self, inst: u32) -> bool {
        inst & 0xfc00_707f == 0x4000_5013
    }

    #[inline(always)]
    #[cfg_attr(feature = "trace", tracing::instrument(name = "SRAI", skip_all, fields(rd = tracing::field::Empty, rs1 = tracing::field::Empty, shamt = tracing::field::Empty)))]
    fn call(&self, inst: u32, hart: &mut Hart, _bus: &mut dyn Bus) -> anyhow::Result<()> {
        let rd = ((inst >> 7) & 0x1f) as usize;
        let rs1 = ((inst >> 15) & 0x1f) as usize;
        let shamt = (inst >> 20) & 0x3f;

        #[cfg(feature = "trace")]
        {
            let span = tracing::Span::current();
            span.record("rd", Hart::IABI[rd]);
            span.record("rs1", Hart::IABI[rs1]);
            span.record("shamt", shamt);
        }

        let val = ((hart.xreg(rs1) as i64) >> shamt) as u64;

        hart.set_xreg(rd, val);
        hart.next_pc();

        Ok(())
    }
}
