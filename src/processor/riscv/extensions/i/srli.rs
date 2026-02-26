use crate::{
    memory::Bus,
    processor::riscv::{hart::Hart, instruction::InstrExec},
};

#[derive(Debug)]
pub struct Srli;

impl InstrExec for Srli {
    #[inline(always)]
    fn matches(&self, inst: u32) -> bool {
        inst & 0xfc00_707f == 0x5013
    }

    #[inline(always)]
    #[cfg_attr(feature = "trace", tracing::instrument(name = "SRLI", skip_all, fields(rd = tracing::field::Empty, rs1 = tracing::field::Empty, shamt = tracing::field::Empty)))]
    fn call(&self, inst: u32, hart: &mut Hart, _bus: &mut dyn Bus) -> anyhow::Result<()> {
        let rd = ((inst >> 7) & 0x1f) as usize;
        let rs1 = ((inst >> 15) & 0x1f) as usize;
        let shamt = (inst >> 20) & 0x3f;

        #[cfg(feature = "trace")]
        {
            let span = tracing::Span::current();
            span.record("rd", Hart::IABI[rd]);
            span.record("rs1", Hart::IABI[rs1]);
            span.record("shamt", shamt);
        }

        let val = hart.xreg(rs1) >> shamt;
        hart.set_xreg(rd, val);
        hart.next_pc();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::mmap::Mmap;
    use crate::processor::riscv::hart::Hart;

    fn setup() -> (Hart, Mmap) {
        (Hart::new(0), Mmap::new(0x0, 0x10_0000))
    }

    fn encode_srli(rd: usize, rs1: usize, shamt: u32) -> u32 {
        ((shamt & 0x3f) << 20)
            | ((rs1 as u32) << 15)
            | (0b101 << 12)
            | ((rd as u32) << 7)
            | 0b0010011
    }

    fn exec(inst: u32, hart: &mut Hart, bus: &mut Mmap) {
        let instr = Srli;
        instr
            .call(inst, hart, bus)
            .expect("SRLI execution unexpectedly trapped");
    }

    #[test]
    fn srli_basic() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 16);
        exec(encode_srli(2, 1, 2), &mut hart, &mut bus);
        assert_eq!(hart.xreg(2), 4);
    }

    #[test]
    fn srli_zero() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 0);
        exec(encode_srli(2, 1, 5), &mut hart, &mut bus);
        assert_eq!(hart.xreg(2), 0);
    }

    #[test]
    fn srli_large_shift() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 0x8000000000000000);
        exec(encode_srli(2, 1, 63), &mut hart, &mut bus);
        assert_eq!(hart.xreg(2), 1);
    }

    #[test]
    fn srli_self() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 8);
        exec(encode_srli(1, 1, 1), &mut hart, &mut bus);
        assert_eq!(hart.xreg(1), 4);
    }

    #[test]
    fn srli_all_ones() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, u64::MAX);
        exec(encode_srli(2, 1, 4), &mut hart, &mut bus);
        assert_eq!(hart.xreg(2), u64::MAX >> 4);
    }
}
