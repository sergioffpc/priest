use crate::{
    memory::Bus,
    processor::riscv::{hart::Hart, instruction::InstrExec},
};

#[derive(Debug)]
pub struct Slli;

impl InstrExec for Slli {
    #[inline(always)]
    fn matches(&self, inst: u32) -> bool {
        inst & 0xfc00_707f == 0x1013
    }

    #[inline(always)]
    #[cfg_attr(feature = "trace", tracing::instrument(name = "SLLI", skip_all, fields(rd = tracing::field::Empty, rs1 = tracing::field::Empty, shamt = tracing::field::Empty)))]
    fn call(&self, inst: u32, hart: &mut Hart, _bus: &mut dyn Bus) -> anyhow::Result<()> {
        let rd = ((inst >> 7) & 0x1f) as usize;
        let rs1 = ((inst >> 15) & 0x1f) as usize;
        let shamt = (inst >> 20) & 0x3f;

        #[cfg(feature = "trace")]
        {
            let span = tracing::Span::current();
            span.record("rd", Hart::IABI[rd]);
            span.record("rs1", Hart::IABI[rs1]);
            span.record("shamt", shamt);
        }

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

    fn encode_slli(rd: u32, rs1: u32, shamt: u32) -> u32 {
        (shamt << 20) | (rs1 << 15) | (0b001 << 12) | (rd << 7) | 0b0010011
    }

    fn setup() -> (Hart, Mmap) {
        (Hart::new(0), Mmap::new(0x0, 0x10_0000))
    }

    fn exec(inst: u32, hart: &mut Hart, bus: &mut Mmap) {
        Slli.call(inst, hart, bus)
            .expect("SLLI execution unexpectedly trapped");
    }

    #[test]
    fn slli_basic() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 0x1);

        exec(encode_slli(2, 1, 4), &mut hart, &mut bus);
        assert_eq!(hart.xreg(2), 0x10);
    }

    #[test]
    fn slli_zero_shift() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 0x1234);

        exec(encode_slli(2, 1, 0), &mut hart, &mut bus);
        assert_eq!(hart.xreg(2), 0x1234);
    }

    #[test]
    fn slli_large_shift() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 1);

        exec(encode_slli(2, 1, 63), &mut hart, &mut bus);
        assert_eq!(hart.xreg(2), 1u64 << 63);
    }

    #[test]
    fn slli_rd_x0() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 0xF);

        exec(encode_slli(0, 1, 2), &mut hart, &mut bus);
        assert_eq!(hart.xreg(0), 0);
    }
}
