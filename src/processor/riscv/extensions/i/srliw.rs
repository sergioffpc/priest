use crate::{
    memory::Bus,
    processor::riscv::{hart::Hart, instruction::InstrExec},
};

#[derive(Debug)]
pub struct Srliw;

impl InstrExec for Srliw {
    #[inline(always)]
    fn matches(&self, inst: u32) -> bool {
        inst & 0xfe00_707f == 0x501b
    }

    #[inline(always)]
    #[cfg_attr(feature = "trace", tracing::instrument(name = "SRLIW", skip_all, fields(rd = tracing::field::Empty, rs1 = tracing::field::Empty, shamt = tracing::field::Empty)))]
    fn call(&self, inst: u32, hart: &mut Hart, _bus: &mut dyn Bus) -> anyhow::Result<()> {
        let rd = ((inst >> 7) & 0x1f) as usize;
        let rs1 = ((inst >> 15) & 0x1f) as usize;
        let shamt = (inst >> 20) & 0x1f;

        #[cfg(feature = "trace")]
        {
            let span = tracing::Span::current();
            span.record("rd", Hart::IABI[rd]);
            span.record("rs1", Hart::IABI[rs1]);
            span.record("shamt", shamt);
        }

        let val = (hart.xreg(rs1) as u32).wrapping_shr(shamt) as i32 as i64;

        hart.set_xreg(rd, val as u64);
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

    fn encode_srliw(rd: usize, rs1: usize, shamt: u32) -> u32 {
        ((shamt & 0x1f) << 20)
            | ((rs1 as u32) << 15)
            | (0b101 << 12)
            | ((rd as u32) << 7)
            | 0b0011011
    }

    fn exec(inst: u32, hart: &mut Hart, bus: &mut Mmap) {
        let instr = Srliw;
        instr
            .call(inst, hart, bus)
            .expect("SRLIW execution unexpectedly trapped");
    }

    #[test]
    fn srliw_basic() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 16);
        exec(encode_srliw(2, 1, 2), &mut hart, &mut bus);
        assert_eq!(hart.xreg(2), 4);
    }

    #[test]
    fn srliw_zero() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 0);
        exec(encode_srliw(2, 1, 5), &mut hart, &mut bus);
        assert_eq!(hart.xreg(2), 0);
    }

    #[test]
    fn srliw_all_ones() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, u64::MAX);
        exec(encode_srliw(2, 1, 4), &mut hart, &mut bus);
        assert_eq!(hart.xreg(2), (u32::MAX >> 4) as u64);
    }

    #[test]
    fn srliw_self() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 8);
        exec(encode_srliw(1, 1, 1), &mut hart, &mut bus);
        assert_eq!(hart.xreg(1), 4);
    }

    #[test]
    fn srliw_large_shift() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 0x80000000);
        exec(encode_srliw(3, 1, 31), &mut hart, &mut bus);
        assert_eq!(hart.xreg(3), 1);
    }
}
