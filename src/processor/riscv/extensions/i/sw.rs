use crate::{
    memory::Bus,
    processor::riscv::{hart::Hart, instruction::InstrExec},
};

#[derive(Debug)]
pub struct Sw;

impl InstrExec for Sw {
    #[inline(always)]
    fn matches(&self, inst: u32) -> bool {
        inst & 0x707f == 0x2023
    }

    #[inline(always)]
    #[cfg_attr(feature = "trace", tracing::instrument(name = "SW", skip_all, fields(rd = tracing::field::Empty, rs1 = tracing::field::Empty, imm = tracing::field::Empty)))]
    fn call(&self, inst: u32, hart: &mut Hart, bus: &mut dyn Bus) -> anyhow::Result<()> {
        let rs1 = ((inst >> 15) & 0x1f) as usize;
        let rs2 = ((inst >> 20) & 0x1f) as usize;
        let imm = ((((inst >> 7) & 0x1f) | (((inst >> 25) & 0x7f) << 5)) as i32) << 20 >> 20;

        #[cfg(feature = "trace")]
        {
            let span = tracing::Span::current();
            span.record("rs1", Hart::IABI[rs1]);
            span.record("rs2", Hart::IABI[rs2]);
            span.record("imm", imm);
        }

        let addr = hart.xreg(rs1).wrapping_add_signed(imm as i64);
        bus.write32(addr, hart.xreg(rs2) as u32)?;
        hart.next_pc();

        Ok(())
    }
}
