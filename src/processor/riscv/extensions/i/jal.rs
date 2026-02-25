use crate::{
    memory::Bus,
    processor::riscv::{hart::Hart, instruction::InstrExec},
};

#[derive(Debug)]
pub struct Jal;

impl InstrExec for Jal {
    #[inline(always)]
    fn matches(&self, inst: u32) -> bool {
        inst & 0x7f == 0x6f
    }

    #[inline(always)]
    #[cfg_attr(feature = "trace", tracing::instrument(name = "JAL", skip_all, fields(rd = tracing::field::Empty, imm = tracing::field::Empty)))]
    fn call(&self, inst: u32, hart: &mut Hart, _bus: &mut dyn Bus) -> anyhow::Result<()> {
        let rd = ((inst >> 7) & 0x1f) as usize;

        let mut imm = ((inst >> 31) & 0x1) << 20;
        imm |= ((inst >> 12) & 0xff) << 12;
        imm |= ((inst >> 20) & 0x1) << 11;
        imm |= ((inst >> 21) & 0x3ff) << 1;
        let imm = ((imm as i32) << 11) as i64 >> 11;

        #[cfg(feature = "trace")]
        {
            let span = tracing::Span::current();
            span.record("rd", Hart::IABI[rd]);
            span.record("imm", imm);
        }

        let pc = hart.pc();
        hart.set_xreg(rd, pc.wrapping_add(Hart::ILEN));

        let target = pc.wrapping_add_signed(imm);
        if target & 0b11 != 0 {
            return Err(crate::memory::exception::Trap::MisalignedFetch { addr: target }.into());
        }
        hart.set_pc(target);

        Ok(())
    }
}
