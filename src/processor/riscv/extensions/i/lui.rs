use crate::{
    memory::Bus,
    processor::riscv::{hart::Hart, instruction::InstrExec},
};

#[derive(Debug)]
pub struct Lui;

impl InstrExec for Lui {
    #[inline(always)]
    fn matches(&self, inst: u32) -> bool {
        inst & 0x7f == 0x37
    }

    #[inline(always)]
    #[cfg_attr(feature = "trace", tracing::instrument(name = "LUI", skip_all, fields(rd = tracing::field::Empty, imm = tracing::field::Empty)))]
    fn call(&self, inst: u32, hart: &mut Hart, _bus: &mut dyn Bus) -> anyhow::Result<()> {
        let rd = ((inst >> 7) & 0x1f) as usize;
        let imm = (inst & 0xfffff000) as i64;

        #[cfg(feature = "trace")]
        {
            let span = tracing::Span::current();
            span.record("rd", Hart::IABI[rd]);
            span.record("imm", imm);
        }

        hart.set_xreg(rd, imm as u64);
        hart.next_pc();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        memory::mmap::Mmap,
        processor::riscv::{hart::Hart, instruction::InstrExec},
    };

    fn encode_lui(rd: u32, imm: u32) -> u32 {
        (imm << 12) | (rd << 7) | 0b0110111
    }

    fn setup() -> (Hart, Mmap) {
        (Hart::new(0), Mmap::new(0x0, 0x10_0000))
    }

    fn exec(inst: u32, hart: &mut Hart, bus: &mut Mmap) {
        Lui.call(inst, hart, bus)
            .expect("LUI execution unexpectedly trapped");
    }

    #[test]
    fn lui_basic() {
        let (mut hart, mut bus) = setup();
        exec(encode_lui(1, 0x12345), &mut hart, &mut bus);
        assert_eq!(hart.xreg(1), 0x12345000);
    }

    #[test]
    fn lui_rd_x0() {
        let (mut hart, mut bus) = setup();
        exec(encode_lui(0, 0xABCDE), &mut hart, &mut bus);
        assert_eq!(hart.xreg(0), 0);
    }

    #[test]
    fn lui_max_value() {
        let (mut hart, mut bus) = setup();
        exec(encode_lui(2, 0xfffff), &mut hart, &mut bus);
        assert_eq!(hart.xreg(2), 0xfffff000);
    }

    #[test]
    fn lui_zero_value() {
        let (mut hart, mut bus) = setup();
        exec(encode_lui(3, 0), &mut hart, &mut bus);
        assert_eq!(hart.xreg(3), 0);
    }
}
