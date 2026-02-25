use crate::{
    memory::Bus,
    processor::riscv::{hart::Hart, instruction::InstrExec},
};

#[derive(Debug)]
pub struct Sltu;

impl InstrExec for Sltu {
    #[inline(always)]
    fn matches(&self, inst: u32) -> bool {
        inst & 0xfe00_707f == 0x3033
    }

    #[inline(always)]
    #[cfg_attr(feature = "trace", tracing::instrument(name = "SLTU", skip_all, fields(rd = tracing::field::Empty, rs1 = tracing::field::Empty, rs2 = tracing::field::Empty)))]
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

        let val = if hart.xreg(rs1) < hart.xreg(rs2) {
            1
        } else {
            0
        };
        hart.set_xreg(rd, val);
        hart.next_pc();

        Ok(())
    }
}
