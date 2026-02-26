use crate::{
    memory::Bus,
    processor::riscv::{hart::Hart, instruction::InstrExec},
};

#[derive(Debug)]
pub struct Addw;

impl InstrExec for Addw {
    #[inline(always)]
    fn matches(&self, inst: u32) -> bool {
        inst & 0xfe00707f == 0x3b
    }

    #[inline(always)]
    #[cfg_attr(feature = "trace", tracing::instrument(name = "ADDW", skip_all, fields(rd = tracing::field::Empty, rs1 = tracing::field::Empty, rs2 = tracing::field::Empty)))]
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

        let val = (hart.xreg(rs1) as i32).wrapping_add(hart.xreg(rs2) as i32) as i64;

        hart.set_xreg(rd, val as u64);
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

    fn encode_addw(rd: u32, rs1: u32, rs2: u32) -> u32 {
        (0b0000000 << 25) | (rs2 << 20) | (rs1 << 15) | (0b000 << 12) | (rd << 7) | 0b0111011
    }

    fn setup() -> (Hart, Mmap) {
        (Hart::new(0), Mmap::new(0x0, 0x10_0000))
    }

    fn exec(inst: u32, hart: &mut Hart, bus: &mut Mmap) {
        Addw.call(inst, hart, bus)
            .expect("ADDW execution unexpectedly trapped");
    }

    #[test]
    fn addw_basic() {
        let (mut hart, mut bus) = setup();

        hart.set_xreg(1, 1);
        hart.set_xreg(2, 2);

        exec(encode_addw(3, 1, 2), &mut hart, &mut bus);

        assert_eq!(hart.xreg(3), 3);
    }

    #[test]
    fn addw_sign_extend_positive() {
        let (mut hart, mut bus) = setup();

        hart.set_xreg(1, 0x000000007fffffff);
        hart.set_xreg(2, 0);

        exec(encode_addw(3, 1, 2), &mut hart, &mut bus);

        assert_eq!(hart.xreg(3), 0x7fffffff);
    }

    #[test]
    fn addw_sign_extend_negative() {
        let (mut hart, mut bus) = setup();

        hart.set_xreg(1, 0x80000000);
        hart.set_xreg(2, 0);

        exec(encode_addw(3, 1, 2), &mut hart, &mut bus);

        assert_eq!(hart.xreg(3), 0xffffffff80000000);
    }

    #[test]
    fn addw_overflow_wrap_32bit() {
        let (mut hart, mut bus) = setup();

        hart.set_xreg(1, 0x7fffffff);
        hart.set_xreg(2, 1);

        exec(encode_addw(3, 1, 2), &mut hart, &mut bus);

        assert_eq!(hart.xreg(3), 0xffffffff80000000);
    }

    #[test]
    fn addw_negative_result() {
        let (mut hart, mut bus) = setup();

        hart.set_xreg(1, 5);
        hart.set_xreg(2, (-10i32 as i64) as u64);

        exec(encode_addw(3, 1, 2), &mut hart, &mut bus);

        assert_eq!(hart.xreg(3), (-5i32 as i64) as u64);
    }

    #[test]
    fn addw_rd_rs1_same() {
        let (mut hart, mut bus) = setup();

        hart.set_xreg(1, 100);
        hart.set_xreg(2, 23);

        exec(encode_addw(1, 1, 2), &mut hart, &mut bus);

        assert_eq!(hart.xreg(1), 123);
    }

    #[test]
    fn addw_rd_rs2_same() {
        let (mut hart, mut bus) = setup();

        hart.set_xreg(1, 100);
        hart.set_xreg(2, 23);

        exec(encode_addw(2, 1, 2), &mut hart, &mut bus);

        assert_eq!(hart.xreg(2), 123);
    }

    #[test]
    fn addw_rs1_rs2_same() {
        let (mut hart, mut bus) = setup();

        hart.set_xreg(1, 7);

        exec(encode_addw(3, 1, 1), &mut hart, &mut bus);

        assert_eq!(hart.xreg(3), 14);
    }

    #[test]
    fn addw_x0_source() {
        let (mut hart, mut bus) = setup();

        hart.set_xreg(1, (-1i32 as i64) as u64);

        exec(encode_addw(3, 1, 0), &mut hart, &mut bus);

        assert_eq!(hart.xreg(3), 0xffffffffffffffff);
    }

    #[test]
    fn addw_x0_destination() {
        let (mut hart, mut bus) = setup();

        hart.set_xreg(1, 5);
        hart.set_xreg(2, 6);

        exec(encode_addw(0, 1, 2), &mut hart, &mut bus);

        assert_eq!(hart.xreg(0), 0);
    }

    #[test]
    fn addw_ignores_upper_32bits() {
        let (mut hart, mut bus) = setup();

        hart.set_xreg(1, 0xffffffff00000001);
        hart.set_xreg(2, 1);

        exec(encode_addw(3, 1, 2), &mut hart, &mut bus);

        assert_eq!(hart.xreg(3), 2);
    }
}
