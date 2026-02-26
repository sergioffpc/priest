use crate::{
    memory::Bus,
    processor::riscv::{hart::Hart, instruction::InstrExec},
};

#[derive(Debug)]
pub struct Sllw;

impl InstrExec for Sllw {
    #[inline(always)]
    fn matches(&self, inst: u32) -> bool {
        inst & 0xfe00_707f == 0x103b
    }

    #[inline(always)]
    #[cfg_attr(feature = "trace", tracing::instrument(name = "SLLW", skip_all, fields(rd = tracing::field::Empty, rs1 = tracing::field::Empty, rs2 = tracing::field::Empty)))]
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
        let val = hart.xreg(rs1).wrapping_shl(shamt);
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

    fn encode_sllw(rd: u32, rs1: u32, rs2: u32) -> u32 {
        (0b0000000 << 25) | (rs2 << 20) | (rs1 << 15) | (0b001 << 12) | (rd << 7) | 0b0111011
    }

    fn setup() -> (Hart, Mmap) {
        (Hart::new(0), Mmap::new(0x0, 0x10_0000))
    }

    fn exec(inst: u32, hart: &mut Hart, bus: &mut Mmap) {
        Sllw.call(inst, hart, bus)
            .expect("SLLW execution unexpectedly trapped");
    }

    #[test]
    fn sllw_basic() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 0x1);
        hart.set_xreg(2, 4);

        exec(encode_sllw(3, 1, 2), &mut hart, &mut bus);
        assert_eq!(hart.xreg(3), 0x10);
    }

    #[test]
    fn sllw_zero() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 0);
        hart.set_xreg(2, 5);

        exec(encode_sllw(3, 1, 2), &mut hart, &mut bus);
        assert_eq!(hart.xreg(3), 0);
    }

    #[test]
    fn sllw_self() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 0x3);
        hart.set_xreg(2, 1);

        exec(encode_sllw(1, 1, 2), &mut hart, &mut bus);
        assert_eq!(hart.xreg(1), 0x6);
    }

    #[test]
    fn sllw_large_shift() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 1);
        hart.set_xreg(2, 31);

        exec(encode_sllw(3, 1, 2), &mut hart, &mut bus);
        assert_eq!(hart.xreg(3), 1u64 << 31);
    }

    #[test]
    fn sllw_rd_x0() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 0xF);
        hart.set_xreg(2, 2);

        exec(encode_sllw(0, 1, 2), &mut hart, &mut bus);
        assert_eq!(hart.xreg(0), 0);
    }
}
