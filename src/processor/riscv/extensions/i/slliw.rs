use crate::{
    memory::Bus,
    processor::riscv::{hart::Hart, instruction::InstrExec},
};

#[derive(Debug)]
pub struct Slliw;

impl InstrExec for Slliw {
    #[inline(always)]
    fn matches(&self, inst: u32) -> bool {
        inst & 0xfe00_707f == 0x101b
    }

    #[inline(always)]
    #[cfg_attr(feature = "trace", tracing::instrument(name = "SLLIW", skip_all, fields(rd = tracing::field::Empty, rs1 = tracing::field::Empty, shamt = tracing::field::Empty)))]
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

        let val = ((hart.xreg(rs1) as u32).wrapping_shl(shamt)) as i32 as i64;
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

    fn encode_slliw(rd: usize, rs1: usize, shamt: u32) -> u32 {
        ((shamt & 0x1f) << 20)
            | ((rs1 as u32) << 15)
            | (0b001 << 12)
            | ((rd as u32) << 7)
            | 0b0011011
    }

    fn exec(inst: u32, hart: &mut Hart, bus: &mut Mmap) {
        let instr = Slliw;
        instr
            .call(inst, hart, bus)
            .expect("SLLIW execution unexpectedly trapped");
    }

    #[test]
    fn slliw_basic() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 1);
        exec(encode_slliw(2, 1, 4), &mut hart, &mut bus);
        assert_eq!(hart.xreg(2), 16);
    }

    #[test]
    fn slliw_zero() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 0x12345678);
        exec(encode_slliw(2, 1, 0), &mut hart, &mut bus);
        assert_eq!(hart.xreg(2), 0x12345678);
    }

    #[test]
    fn slliw_sign_extend() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 0x40000000);
        exec(encode_slliw(2, 1, 1), &mut hart, &mut bus);
        assert_eq!(hart.xreg(2), 0xffffffff80000000);
    }

    #[test]
    fn slliw_self() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 3);
        exec(encode_slliw(1, 1, 1), &mut hart, &mut bus);
        assert_eq!(hart.xreg(1), 6);
    }

    #[test]
    fn slliw_large_shift() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 1);
        exec(encode_slliw(2, 1, 31), &mut hart, &mut bus);
        assert_eq!(hart.xreg(2), 0xffffffff80000000);
    }
}
