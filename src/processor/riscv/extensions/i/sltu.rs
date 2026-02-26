use crate::{
    memory::Bus,
    processor::riscv::{hart::Hart, instruction::InstrExec},
};

#[derive(Debug)]
pub struct Sltu;

impl InstrExec for Sltu {
    #[inline(always)]
    fn matches(&self, inst: u32) -> bool {
        inst & 0xfe00_707f == 0x3033
    }

    #[inline(always)]
    #[cfg_attr(feature = "trace", tracing::instrument(name = "SLTU", skip_all, fields(rd = tracing::field::Empty, rs1 = tracing::field::Empty, rs2 = tracing::field::Empty)))]
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

        let val = if hart.xreg(rs1) < hart.xreg(rs2) {
            1
        } else {
            0
        };
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

    fn encode_sltu(rd: usize, rs1: usize, rs2: usize) -> u32 {
        ((rs2 as u32) << 20) | ((rs1 as u32) << 15) | (0b011 << 12) | ((rd as u32) << 7) | 0b0110011
    }

    fn exec(inst: u32, hart: &mut Hart, bus: &mut Mmap) {
        let instr = Sltu;
        instr
            .call(inst, hart, bus)
            .expect("SLTU execution unexpectedly trapped");
    }

    #[test]
    fn sltu_true() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 5);
        hart.set_xreg(2, 10);
        exec(encode_sltu(3, 1, 2), &mut hart, &mut bus);
        assert_eq!(hart.xreg(3), 1);
    }

    #[test]
    fn sltu_false() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 10);
        hart.set_xreg(2, 5);
        exec(encode_sltu(3, 1, 2), &mut hart, &mut bus);
        assert_eq!(hart.xreg(3), 0);
    }

    #[test]
    fn sltu_equal() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 7);
        hart.set_xreg(2, 7);
        exec(encode_sltu(3, 1, 2), &mut hart, &mut bus);
        assert_eq!(hart.xreg(3), 0);
    }

    #[test]
    fn sltu_signed_vs_unsigned() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, (-1i64) as u64);
        hart.set_xreg(2, 1);
        exec(encode_sltu(3, 1, 2), &mut hart, &mut bus);
        assert_eq!(hart.xreg(3), 0);
    }

    #[test]
    fn sltu_unsigned_wrap() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 0);
        hart.set_xreg(2, u64::MAX);
        exec(encode_sltu(3, 1, 2), &mut hart, &mut bus);
        assert_eq!(hart.xreg(3), 1);
    }

    #[test]
    fn sltu_self() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 123456);
        exec(encode_sltu(1, 1, 1), &mut hart, &mut bus);
        assert_eq!(hart.xreg(1), 0);
    }
}
