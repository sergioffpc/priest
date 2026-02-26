use crate::{
    memory::Bus,
    processor::riscv::{hart::Hart, instruction::InstrExec},
};

#[derive(Debug)]
pub struct Or;

impl InstrExec for Or {
    #[inline(always)]
    fn matches(&self, inst: u32) -> bool {
        inst & 0xfe00_707f == 0x6033
    }

    #[inline(always)]
    #[cfg_attr(feature = "trace", tracing::instrument(name = "OR", skip_all, fields(rd = tracing::field::Empty, rs1 = tracing::field::Empty, rs2 = tracing::field::Empty)))]
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

        let val = hart.xreg(rs1) | hart.xreg(rs2);
        hart.set_xreg(rd, val);
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

    fn encode_or(rd: u32, rs1: u32, rs2: u32) -> u32 {
        (0b0000000 << 25) | (rs2 << 20) | (rs1 << 15) | (0b110 << 12) | (rd << 7) | 0b0110011
    }

    fn setup() -> (Hart, Mmap) {
        (Hart::new(0), Mmap::new(0x0, 0x10_0000))
    }

    fn exec(inst: u32, hart: &mut Hart, bus: &mut Mmap) {
        Or.call(inst, hart, bus)
            .expect("OR execution unexpectedly trapped");
    }

    #[test]
    fn or_basic() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 0x1234);
        hart.set_xreg(2, 0x00ff);

        exec(encode_or(3, 1, 2), &mut hart, &mut bus);
        assert_eq!(hart.xreg(3), 0x12ff);
    }

    #[test]
    fn or_zero() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 0);
        hart.set_xreg(2, 0);

        exec(encode_or(3, 1, 2), &mut hart, &mut bus);
        assert_eq!(hart.xreg(3), 0);
    }

    #[test]
    fn or_self() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 0x5555);

        exec(encode_or(1, 1, 1), &mut hart, &mut bus);
        assert_eq!(hart.xreg(1), 0x5555);
    }

    #[test]
    fn or_x0_source() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 0x1234);

        exec(encode_or(3, 1, 0), &mut hart, &mut bus);
        assert_eq!(hart.xreg(3), 0x1234);
    }

    #[test]
    fn or_x0_destination() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 0x1234);
        hart.set_xreg(2, 0x5678);

        exec(encode_or(0, 1, 2), &mut hart, &mut bus);
        assert_eq!(hart.xreg(0), 0);
    }

    #[test]
    fn or_large_values() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 0x8000000000000000);
        hart.set_xreg(2, 0x7fffffffffffffff);

        exec(encode_or(3, 1, 2), &mut hart, &mut bus);
        assert_eq!(hart.xreg(3), 0xffffffffffffffff);
    }
}
