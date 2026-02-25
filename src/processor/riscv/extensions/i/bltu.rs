use crate::{
    memory::Bus,
    processor::riscv::{hart::Hart, instruction::InstrExec},
};

#[derive(Debug)]
pub struct Bltu;

impl InstrExec for Bltu {
    #[inline(always)]
    fn matches(&self, inst: u32) -> bool {
        inst & 0x707f == 0x6063
    }

    #[inline(always)]
    #[cfg_attr(feature = "trace", tracing::instrument(name = "BLTU", skip_all, fields(rs1 = tracing::field::Empty, rs2 = tracing::field::Empty, imm = tracing::field::Empty)))]
    fn call(&self, inst: u32, hart: &mut Hart, _bus: &mut dyn Bus) -> anyhow::Result<()> {
        let rs1 = ((inst >> 15) & 0x1f) as usize;
        let rs2 = ((inst >> 20) & 0x1f) as usize;

        let mut imm = ((inst >> 31) & 0x1) << 12;
        imm |= ((inst >> 7) & 0x1) << 11;
        imm |= ((inst >> 25) & 0x3f) << 5;
        imm |= ((inst >> 8) & 0xf) << 1;
        let imm = ((imm as i16) << 3) as i64 >> 3;

        #[cfg(feature = "trace")]
        {
            let span = tracing::Span::current();
            span.record("rs1", Hart::IABI[rs1]);
            span.record("rs2", Hart::IABI[rs2]);
            span.record("imm", imm);
        }

        if hart.xreg(rs1) < hart.xreg(rs2) {
            let target = hart.pc().wrapping_add_signed(imm);
            if target & 0b11 != 0 {
                return Err(
                    crate::memory::exception::Trap::MisalignedFetch { addr: target }.into(),
                );
            }
            hart.set_pc(target);
        } else {
            hart.next_pc();
        }

        Ok(())
    }
}
