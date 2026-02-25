use crate::{
    memory::Bus,
    processor::riscv::{hart::Hart, instruction::InstrExec},
};

#[derive(Debug)]
pub struct Sd;

impl InstrExec for Sd {
    #[inline(always)]
    fn matches(&self, inst: u32) -> bool {
        inst & 0x707f == 0x3023
    }

    #[inline(always)]
    #[cfg_attr(feature = "trace", tracing::instrument(name = "SD", skip_all, fields(rs1 = tracing::field::Empty, rs2 = tracing::field::Empty, imm = tracing::field::Empty)))]
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
        bus.write64(addr, hart.xreg(rs2))?;
        hart.next_pc();

        Ok(())
    }
}
