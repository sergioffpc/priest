use crate::{
    memory::Bus,
    processor::riscv::{hart::Hart, instruction::InstrExec},
};

#[derive(Debug)]
pub struct And;

impl InstrExec for And {
    #[inline(always)]
    fn matches(&self, inst: u32) -> bool {
        inst & 0xfe00_707f == 0x7033
    }

    #[inline(always)]
    #[cfg_attr(feature = "trace", tracing::instrument(name = "AND", skip_all, fields(rd = tracing::field::Empty, rs1 = tracing::field::Empty, rs2 = tracing::field::Empty)))]
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

        hart.set_xreg(rd, hart.xreg(rs1) & hart.xreg(rs2));
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

    fn encode_and(rd: u32, rs1: u32, rs2: u32) -> u32 {
        (0b0000000 << 25) | (rs2 << 20) | (rs1 << 15) | (0b111 << 12) | (rd << 7) | 0b0110011
    }

    fn setup() -> (Hart, Mmap) {
        (Hart::new(0), Mmap::new(0x0, 0x10_0000))
    }

    fn exec(inst: u32, hart: &mut Hart, bus: &mut Mmap) {
        And.call(inst, hart, bus)
            .expect("AND execution unexpectedly trapped");
    }

    #[test]
    fn and_basic() {
        let (mut hart, mut bus) = setup();

        hart.set_xreg(1, 0b1100);
        hart.set_xreg(2, 0b1010);

        exec(encode_and(3, 1, 2), &mut hart, &mut bus);

        assert_eq!(hart.xreg(3), 0b1000);
    }

    #[test]
    fn and_zero() {
        let (mut hart, mut bus) = setup();

        hart.set_xreg(1, 0b1111);
        hart.set_xreg(2, 0);

        exec(encode_and(3, 1, 2), &mut hart, &mut bus);

        assert_eq!(hart.xreg(3), 0);
    }

    #[test]
    fn and_all_ones() {
        let (mut hart, mut bus) = setup();

        hart.set_xreg(1, u64::MAX);
        hart.set_xreg(2, u64::MAX);

        exec(encode_and(3, 1, 2), &mut hart, &mut bus);

        assert_eq!(hart.xreg(3), u64::MAX);
    }

    #[test]
    fn and_mask() {
        let (mut hart, mut bus) = setup();

        hart.set_xreg(1, 0xffffffffffffffff);
        hart.set_xreg(2, 0x0f0f0f0f0f0f0f0f);

        exec(encode_and(3, 1, 2), &mut hart, &mut bus);

        assert_eq!(hart.xreg(3), 0x0f0f0f0f0f0f0f0f);
    }

    #[test]
    fn and_rd_rs1_same() {
        let (mut hart, mut bus) = setup();

        hart.set_xreg(1, 0b1010);
        hart.set_xreg(2, 0b1100);

        exec(encode_and(1, 1, 2), &mut hart, &mut bus);

        assert_eq!(hart.xreg(1), 0b1000);
    }

    #[test]
    fn and_rd_rs2_same() {
        let (mut hart, mut bus) = setup();

        hart.set_xreg(1, 0b1010);
        hart.set_xreg(2, 0b1100);

        exec(encode_and(2, 1, 2), &mut hart, &mut bus);

        assert_eq!(hart.xreg(2), 0b1000);
    }

    #[test]
    fn and_rs1_rs2_same() {
        let (mut hart, mut bus) = setup();

        hart.set_xreg(1, 0x12345678);

        exec(encode_and(3, 1, 1), &mut hart, &mut bus);

        assert_eq!(hart.xreg(3), 0x12345678);
    }

    #[test]
    fn and_x0_source() {
        let (mut hart, mut bus) = setup();

        hart.set_xreg(1, u64::MAX);

        exec(encode_and(3, 1, 0), &mut hart, &mut bus);

        assert_eq!(hart.xreg(3), 0);
    }

    #[test]
    fn and_x0_destination() {
        let (mut hart, mut bus) = setup();

        hart.set_xreg(1, u64::MAX);
        hart.set_xreg(2, u64::MAX);

        exec(encode_and(0, 1, 2), &mut hart, &mut bus);

        assert_eq!(hart.xreg(0), 0);
    }

    #[test]
    fn and_mixed_bits() {
        let (mut hart, mut bus) = setup();

        hart.set_xreg(1, 0xaaaaaaaaaaaaaaaa);
        hart.set_xreg(2, 0x5555555555555555);

        exec(encode_and(3, 1, 2), &mut hart, &mut bus);

        assert_eq!(hart.xreg(3), 0);
    }
}
