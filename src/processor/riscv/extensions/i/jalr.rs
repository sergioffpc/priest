use crate::{
    memory::Bus,
    processor::riscv::{hart::Hart, instruction::InstrExec},
};

#[derive(Debug)]
pub struct Jalr;

impl InstrExec for Jalr {
    #[inline(always)]
    fn matches(&self, inst: u32) -> bool {
        inst & 0x707f == 0x67
    }

    #[inline(always)]
    #[cfg_attr(feature = "trace", tracing::instrument(name = "JALR", skip_all, fields(rd = tracing::field::Empty, rs1 = tracing::field::Empty, imm = tracing::field::Empty)))]
    fn call(&self, inst: u32, hart: &mut Hart, _bus: &mut dyn Bus) -> anyhow::Result<()> {
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

        let pc = hart.pc();
        hart.set_xreg(rd, pc.wrapping_add(Hart::ILEN));

        let target = hart.xreg(rs1).wrapping_add_signed(imm) & !1;
        if target & 0b11 != 0 {
            return Err(crate::memory::exception::Trap::MisalignedFetch { addr: target }.into());
        }
        hart.set_pc(target);

        Ok(())
    }
}
