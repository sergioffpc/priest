use crate::{
    memory::Bus,
    processor::riscv::{hart::Hart, instruction::InstrExec},
};

#[derive(Debug)]
pub struct Beq;

impl InstrExec for Beq {
    #[inline(always)]
    fn matches(&self, inst: u32) -> bool {
        inst & 0x707f == 0x63
    }

    #[inline(always)]
    #[cfg_attr(feature = "trace", tracing::instrument(name = "BEQ", skip_all, fields(rs1 = tracing::field::Empty, rs2 = tracing::field::Empty, imm = tracing::field::Empty)))]
    fn call(&self, inst: u32, hart: &mut Hart, _bus: &mut dyn Bus) -> anyhow::Result<()> {
        let rs1 = ((inst >> 15) & 0x1f) as usize;
        let rs2 = ((inst >> 20) & 0x1f) as usize;
        let imm = {
            let mut val = ((inst >> 31) & 0x1) << 12;
            val |= ((inst >> 7) & 0x1) << 11;
            val |= ((inst >> 25) & 0x3f) << 5;
            val |= ((inst >> 8) & 0xf) << 1;
            ((val as i16) << 3) as i64 >> 3
        };

        #[cfg(feature = "trace")]
        {
            let span = tracing::Span::current();
            span.record("rs1", Hart::IABI[rs1]);
            span.record("rs2", Hart::IABI[rs2]);
            span.record("imm", imm);
        }

        if hart.xreg(rs1) == hart.xreg(rs2) {
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
