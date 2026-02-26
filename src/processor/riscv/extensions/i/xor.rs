use crate::{
    memory::Bus,
    processor::riscv::{hart::Hart, instruction::InstrExec},
};

#[derive(Debug)]
pub struct Xor;

impl InstrExec for Xor {
    #[inline(always)]
    fn matches(&self, inst: u32) -> bool {
        inst & 0xfe00_707f == 0x4033
    }

    #[inline(always)]
    #[cfg_attr(feature = "trace", tracing::instrument(name = "XOR", skip_all, fields(rd = tracing::field::Empty, rs1 = tracing::field::Empty, rs2 = tracing::field::Empty)))]
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

        let val = hart.xreg(rs1) ^ hart.xreg(rs2);
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

    fn encode_xor(rd: usize, rs1: usize, rs2: usize) -> u32 {
        ((0b0000000) << 25)
            | ((rs2 as u32) << 20)
            | ((rs1 as u32) << 15)
            | (0b100 << 12)
            | ((rd as u32) << 7)
            | 0b0110011
    }

    fn exec(inst: u32, hart: &mut Hart, bus: &mut Mmap) {
        let instr = Xor;
        instr
            .call(inst, hart, bus)
            .expect("XOR execution unexpectedly trapped");
    }

    #[test]
    fn xor_basic() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 0b1010);
        hart.set_xreg(2, 0b1100);
        exec(encode_xor(3, 1, 2), &mut hart, &mut bus);
        assert_eq!(hart.xreg(3), 0b0110);
    }

    #[test]
    fn xor_zero() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 0);
        hart.set_xreg(2, 0);
        exec(encode_xor(3, 1, 2), &mut hart, &mut bus);
        assert_eq!(hart.xreg(3), 0);
    }

    #[test]
    fn xor_self() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 0b1111);
        exec(encode_xor(1, 1, 1), &mut hart, &mut bus);
        assert_eq!(hart.xreg(1), 0);
    }

    #[test]
    fn xor_large_values() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 0xffff_ffff_0000_0000);
        hart.set_xreg(2, 0x0000_ffff_ffff_0000);
        exec(encode_xor(3, 1, 2), &mut hart, &mut bus);
        assert_eq!(hart.xreg(3), 0xffff_0000_ffff_0000);
    }
}
