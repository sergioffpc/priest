use crate::{
    memory::Bus,
    processor::riscv::{hart::Hart, instruction::InstrExec},
};

#[derive(Debug)]
pub struct Srlw;

impl InstrExec for Srlw {
    #[inline(always)]
    fn matches(&self, inst: u32) -> bool {
        inst & 0xfe00_707f == 0x503b
    }

    #[inline(always)]
    #[cfg_attr(feature = "trace", tracing::instrument(name = "SRLW", skip_all, fields(rd = tracing::field::Empty, rs1 = tracing::field::Empty, rs2 = tracing::field::Empty)))]
    fn call(&self, inst: u32, hart: &mut Hart, _bus: &mut dyn Bus) -> anyhow::Result<()> {
        let rd = ((inst >> 7) & 0x1f) as usize;
        let rs1 = ((inst >> 15) & 0x1f) as usize;
        let rs2 = ((inst >> 20) & 0x1f) as usize;

        #[cfg(feature = "trace")]
        {
            let span = tracing::Span::current();
            span.record("rd", Hart::IABI[rd]);
            span.record("rs1", Hart::IABI[rs1]);
            span.record("rs2", Hart::IABI[rs2]);
        }

        let shamt = (hart.xreg(rs2) & 0x1f) as u32;
        let val = (hart.xreg(rs1) as u32).wrapping_shr(shamt) as u64;

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

    fn encode_srlw(rd: usize, rs1: usize, rs2: usize) -> u32 {
        (0b0000000 << 25)
            | ((rs2 as u32) << 20)
            | ((rs1 as u32) << 15)
            | (0b101 << 12)
            | ((rd as u32) << 7)
            | 0b0111011
    }

    fn exec(inst: u32, hart: &mut Hart, bus: &mut Mmap) {
        let instr = Srlw;
        instr
            .call(inst, hart, bus)
            .expect("SRLW execution unexpectedly trapped");
    }

    #[test]
    fn srlw_basic() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 16);
        hart.set_xreg(2, 2);
        exec(encode_srlw(3, 1, 2), &mut hart, &mut bus);
        assert_eq!(hart.xreg(3), 4);
    }

    #[test]
    fn srlw_zero() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 0);
        hart.set_xreg(2, 5);
        exec(encode_srlw(3, 1, 2), &mut hart, &mut bus);
        assert_eq!(hart.xreg(3), 0);
    }

    #[test]
    fn srlw_all_ones() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, u64::MAX);
        hart.set_xreg(2, 4);
        exec(encode_srlw(3, 1, 2), &mut hart, &mut bus);
        assert_eq!(hart.xreg(3), (u32::MAX >> 4) as u64);
    }

    #[test]
    fn srlw_self() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 8);
        hart.set_xreg(2, 1);
        exec(encode_srlw(1, 1, 2), &mut hart, &mut bus);
        assert_eq!(hart.xreg(1), 4);
    }

    #[test]
    fn srlw_large_shift() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 0x80000000);
        hart.set_xreg(2, 31);
        exec(encode_srlw(3, 1, 2), &mut hart, &mut bus);
        assert_eq!(hart.xreg(3), 1);
    }
}
