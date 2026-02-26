use crate::{
    memory::Bus,
    processor::riscv::{hart::Hart, instruction::InstrExec},
};

#[derive(Debug)]
pub struct Sd;

impl InstrExec for Sd {
    #[inline(always)]
    fn matches(&self, inst: u32) -> bool {
        inst & 0x707f == 0x3023
    }

    #[inline(always)]
    #[cfg_attr(feature = "trace", tracing::instrument(name = "SD", skip_all, fields(rs1 = tracing::field::Empty, rs2 = tracing::field::Empty, imm = tracing::field::Empty)))]
    fn call(&self, inst: u32, hart: &mut Hart, bus: &mut dyn Bus) -> anyhow::Result<()> {
        let rs1 = ((inst >> 15) & 0x1f) as usize;
        let rs2 = ((inst >> 20) & 0x1f) as usize;
        let imm = ((((inst >> 7) & 0x1f) | (((inst >> 25) & 0x7f) << 5)) as i32) << 20 >> 20;

        #[cfg(feature = "trace")]
        {
            let span = tracing::Span::current();
            span.record("rs1", Hart::IABI[rs1]);
            span.record("rs2", Hart::IABI[rs2]);
            span.record("imm", imm);
        }

        let addr = hart.xreg(rs1).wrapping_add_signed(imm as i64);
        bus.write64(addr, hart.xreg(rs2))?;
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

    fn encode_sd(rs1: u32, rs2: u32, imm: i16) -> u32 {
        let imm12 = (imm as u32) & 0xfff;
        let imm_11_5 = (imm12 >> 5) & 0x7f;
        let imm_4_0 = imm12 & 0x1f;
        (imm_11_5 << 25) | (rs2 << 20) | (rs1 << 15) | (0b011 << 12) | (imm_4_0 << 7) | 0b0100011
    }

    fn setup() -> (Hart, Mmap) {
        (Hart::new(0), Mmap::new(0x0, 0x10_0000))
    }

    fn exec(inst: u32, hart: &mut Hart, bus: &mut Mmap) {
        Sd.call(inst, hart, bus)
            .expect("SD execution unexpectedly trapped");
    }

    #[test]
    fn sd_basic() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 0x100);
        hart.set_xreg(2, 0x1122334455667788);

        exec(encode_sd(1, 2, 0), &mut hart, &mut bus);
        assert_eq!(bus.read64(0x100).unwrap(), 0x1122334455667788);
    }

    #[test]
    fn sd_with_offset() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 0x200);
        hart.set_xreg(2, 0x8877665544332211);

        exec(encode_sd(1, 2, 8), &mut hart, &mut bus);
        assert_eq!(bus.read64(0x208).unwrap(), 0x8877665544332211);
    }

    #[test]
    fn sd_negative_offset() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 0x300);
        hart.set_xreg(2, 0xdeadbeefcafebabe);

        exec(encode_sd(1, 2, -16), &mut hart, &mut bus);
        assert_eq!(bus.read64(0x2F0).unwrap(), 0xdeadbeefcafebabe);
    }

    #[test]
    fn sd_zero_value() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 0x400);
        hart.set_xreg(2, 0);

        exec(encode_sd(1, 2, 0), &mut hart, &mut bus);
        assert_eq!(bus.read64(0x400).unwrap(), 0);
    }
}
