use crate::{
    memory::Bus,
    processor::riscv::{hart::Hart, instruction::InstrExec},
};

#[derive(Debug)]
pub struct Srl;

impl InstrExec for Srl {
    #[inline(always)]
    fn matches(&self, inst: u32) -> bool {
        inst & 0xfe00_707f == 0x5033
    }

    #[inline(always)]
    #[cfg_attr(feature = "trace", tracing::instrument(name = "SRL", skip_all, fields(rd = tracing::field::Empty, rs1 = tracing::field::Empty, rs2 = tracing::field::Empty)))]
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

        let shamt = (hart.xreg(rs2) & 0x3f) as u32;
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

    fn encode_srl(rd: usize, rs1: usize, rs2: usize) -> u32 {
        (0b0000000 << 25)
            | ((rs2 as u32) << 20)
            | ((rs1 as u32) << 15)
            | (0b101 << 12)
            | ((rd as u32) << 7)
            | 0b0110011
    }

    fn exec(inst: u32, hart: &mut Hart, bus: &mut Mmap) {
        let instr = Srl;
        instr
            .call(inst, hart, bus)
            .expect("SRL execution unexpectedly trapped");
    }

    #[test]
    fn srl_basic() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 16);
        hart.set_xreg(2, 2);
        exec(encode_srl(3, 1, 2), &mut hart, &mut bus);
        assert_eq!(hart.xreg(3), 4);
    }

    #[test]
    fn srl_zero() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 0);
        hart.set_xreg(2, 5);
        exec(encode_srl(3, 1, 2), &mut hart, &mut bus);
        assert_eq!(hart.xreg(3), 0);
    }

    #[test]
    fn srl_all_ones() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, u64::MAX);
        hart.set_xreg(2, 4);
        exec(encode_srl(3, 1, 2), &mut hart, &mut bus);
        assert_eq!(hart.xreg(3), u64::MAX >> 4);
    }

    #[test]
    fn srl_self() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 8);
        hart.set_xreg(2, 1);
        exec(encode_srl(1, 1, 2), &mut hart, &mut bus);
        assert_eq!(hart.xreg(1), 4);
    }

    #[test]
    fn srl_large_shift() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 0x8000000000000000);
        hart.set_xreg(2, 63);
        exec(encode_srl(3, 1, 2), &mut hart, &mut bus);
        assert_eq!(hart.xreg(3), 1);
    }
}
