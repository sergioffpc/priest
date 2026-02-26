use crate::{
    memory::Bus,
    processor::riscv::{hart::Hart, instruction::InstrExec},
};

#[derive(Debug)]
pub struct Add;

impl InstrExec for Add {
    #[inline(always)]
    fn matches(&self, inst: u32) -> bool {
        inst & 0xfe00_707f == 0x33
    }

    #[inline(always)]
    #[cfg_attr(feature = "trace", tracing::instrument(name = "ADD", skip_all, fields(rd = tracing::field::Empty, rs1 = tracing::field::Empty, rs2 = tracing::field::Empty)))]
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

        hart.set_xreg(rd, hart.xreg(rs1).wrapping_add(hart.xreg(rs2)));
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

    fn encode_add(rd: u32, rs1: u32, rs2: u32) -> u32 {
        (0b0000000 << 25) | (rs2 << 20) | (rs1 << 15) | (0b000 << 12) | (rd << 7) | 0b0110011
    }

    fn setup() -> (Hart, Mmap) {
        (Hart::new(0), Mmap::new(0x0, 0x10_0000))
    }

    fn exec(inst: u32, hart: &mut Hart, bus: &mut Mmap) {
        Add.call(inst, hart, bus)
            .expect("ADD execution unexpectedly trapped");
    }

    #[test]
    fn add_basic() {
        let (mut hart, mut bus) = setup();

        hart.set_xreg(1, 1);
        hart.set_xreg(2, 2);

        exec(encode_add(3, 1, 2), &mut hart, &mut bus);

        assert_eq!(hart.xreg(3), 3);
    }

    #[test]
    fn add_zero() {
        let (mut hart, mut bus) = setup();

        exec(encode_add(3, 0, 0), &mut hart, &mut bus);

        assert_eq!(hart.xreg(3), 0);
    }

    #[test]
    fn add_negative() {
        let (mut hart, mut bus) = setup();

        hart.set_xreg(1, -1i64 as u64);
        hart.set_xreg(2, 1);

        exec(encode_add(3, 1, 2), &mut hart, &mut bus);

        assert_eq!(hart.xreg(3), 0);
    }

    #[test]
    fn add_overflow_wrap() {
        let (mut hart, mut bus) = setup();

        hart.set_xreg(1, u64::MAX);
        hart.set_xreg(2, 1);

        exec(encode_add(3, 1, 2), &mut hart, &mut bus);

        assert_eq!(hart.xreg(3), 0);
    }

    #[test]
    fn add_self() {
        let (mut hart, mut bus) = setup();

        hart.set_xreg(1, 123);

        exec(encode_add(1, 1, 1), &mut hart, &mut bus);

        assert_eq!(hart.xreg(1), 246);
    }

    #[test]
    fn add_x0_source() {
        let (mut hart, mut bus) = setup();

        hart.set_xreg(1, 5);

        exec(encode_add(3, 1, 0), &mut hart, &mut bus);

        assert_eq!(hart.xreg(3), 5);
    }

    #[test]
    fn add_x0_destination() {
        let (mut hart, mut bus) = setup();

        hart.set_xreg(1, 5);
        hart.set_xreg(2, 6);

        exec(encode_add(0, 1, 2), &mut hart, &mut bus);

        assert_eq!(hart.xreg(0), 0);
    }

    #[test]
    fn add_signed_boundary() {
        let (mut hart, mut bus) = setup();

        hart.set_xreg(1, 0x7fffffffffffffff);
        hart.set_xreg(2, 1);

        exec(encode_add(3, 1, 2), &mut hart, &mut bus);

        assert_eq!(hart.xreg(3), 0x8000000000000000);
    }
}
