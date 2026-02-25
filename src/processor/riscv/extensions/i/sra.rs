use crate::{
    memory::Bus,
    processor::riscv::{hart::Hart, instruction::InstrExec},
};

#[derive(Debug)]
pub struct Sra;

impl InstrExec for Sra {
    #[inline(always)]
    fn matches(&self, inst: u32) -> bool {
        inst & 0xfe00_707f == 0x4000_5033
    }

    #[inline(always)]
    #[cfg_attr(feature = "trace", tracing::instrument(name = "SRA", skip_all, fields(rd = tracing::field::Empty, rs1 = tracing::field::Empty, rs2 = tracing::field::Empty)))]
    fn call(&self, inst: u32, hart: &mut Hart, _bus: &mut dyn Bus) -> anyhow::Result<()> {
        let rd = ((inst >> 7) & 0x1f) as usize;
        let rs1 = ((inst >> 15) & 0x1f) as usize;
        let rs2 = ((inst >> 20) & 0x1f) as usize;

        #[cfg(feature = "trace")]
        {
            let span = tracing::Span::current();
            span.record("rd", Hart::IABI[rd]);
            span.record("rs1", Hart::IABI[rs1]);
            span.record("rs2", Hart::IABI[rs2]);
        }

        let shamt = (hart.xreg(rs2) & 0x3f) as u32;
        let val = (hart.xreg(rs1) as i64 >> shamt) as u64;

        hart.set_xreg(rd, val);
        hart.next_pc();

        Ok(())
    }
}
