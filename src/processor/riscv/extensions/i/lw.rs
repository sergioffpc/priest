use crate::{
    memory::Bus,
    processor::riscv::{hart::Hart, instruction::InstrExec},
};

#[derive(Debug)]
pub struct Lw;

impl InstrExec for Lw {
    #[inline(always)]
    fn matches(&self, inst: u32) -> bool {
        inst & 0x707f == 0x2003
    }

    #[inline(always)]
    #[cfg_attr(feature = "trace", tracing::instrument(name = "LW", skip_all, fields(rd = tracing::field::Empty, rs1 = tracing::field::Empty, imm = tracing::field::Empty)))]
    fn call(&self, inst: u32, hart: &mut Hart, bus: &mut dyn Bus) -> anyhow::Result<()> {
        let rd = ((inst >> 7) & 0x1f) as usize;
        let rs1 = ((inst >> 15) & 0x1f) as usize;
        let imm = ((inst as i32) >> 20) as i64;

        #[cfg(feature = "trace")]
        {
            let span = tracing::Span::current();
            span.record("rd", Hart::IABI[rd]);
            span.record("rs1", Hart::IABI[rs1]);
            span.record("imm", imm);
        }

        let addr = hart.xreg(rs1).wrapping_add_signed(imm);
        let val = (bus.read32(addr)? as i32 as i64) as u64;

        hart.set_xreg(rd, val);
        hart.next_pc();

        Ok(())
    }
}
