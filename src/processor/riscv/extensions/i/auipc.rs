use crate::{
    memory::Bus,
    processor::riscv::{hart::Hart, instruction::InstrExec},
};

#[derive(Debug)]
pub struct Auipc;

impl InstrExec for Auipc {
    #[inline(always)]
    fn matches(&self, inst: u32) -> bool {
        inst & 0x7f == 0x17
    }

    #[inline(always)]
    #[cfg_attr(feature = "trace", tracing::instrument(name = "AUIPC", skip_all, fields(rd = tracing::field::Empty, imm = tracing::field::Empty)))]
    fn call(&self, inst: u32, hart: &mut Hart, _bus: &mut dyn Bus) -> anyhow::Result<()> {
        let rd = ((inst >> 7) & 0x1f) as usize;
        let imm = (inst & 0xfffff000) as u64;

        #[cfg(feature = "trace")]
        {
            let span = tracing::Span::current();
            span.record("rd", Hart::IABI[rd]);
            span.record("imm", imm);
        }

        hart.set_xreg(rd, hart.pc().wrapping_add(imm));
        hart.next_pc();

        Ok(())
    }
}
