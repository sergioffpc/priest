use crate::{
    memory::Bus,
    processor::riscv::{hart::Hart, instruction::InstrExec},
};

#[derive(Debug)]
pub struct Addi;

impl InstrExec for Addi {
    #[inline(always)]
    fn matches(&self, inst: u32) -> bool {
        inst & 0x707f == 0x13
    }

    #[inline(always)]
    #[cfg_attr(feature = "trace", tracing::instrument(name = "ADDI", skip_all, fields(rd = tracing::field::Empty, rs1 = tracing::field::Empty, imm = tracing::field::Empty)))]
    fn call(&self, inst: u32, hart: &mut Hart, _bus: &mut dyn Bus) -> anyhow::Result<()> {
        let rd = ((inst >> 7) & 0x1f) as usize;
        let rs1 = ((inst >> 15) & 0x1f) as usize;

        // Sign-extend 12-bit immediate
        let imm = ((inst as i32) >> 20) as i64 as u64;

        #[cfg(feature = "trace")]
        {
            let span = tracing::Span::current();
            span.record("rd", Hart::IABI[rd]);
            span.record("rs1", Hart::IABI[rs1]);
            span.record("imm", imm);
        }

        hart.set_xreg(rd, hart.xreg(rs1).wrapping_add(imm));
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

    fn encode_addi(rd: u32, rs1: u32, imm: i32) -> u32 {
        let imm12 = (imm as u32) & 0xfff;
        (imm12 << 20) | (rs1 << 15) | (0b000 << 12) | (rd << 7) | 0b0010011
    }

    fn setup() -> (Hart, Mmap) {
        (Hart::new(0), Mmap::new(0x0, 0x10_0000))
    }

    fn exec(inst: u32, hart: &mut Hart, bus: &mut Mmap) {
        Addi.call(inst, hart, bus)
            .expect("ADDI execution unexpectedly trapped");
    }

    #[test]
    fn addi_basic() {
        let (mut hart, mut bus) = setup();

        hart.set_xreg(1, 1);

        exec(encode_addi(3, 1, 2), &mut hart, &mut bus);

        assert_eq!(hart.xreg(3), 3);
    }

    #[test]
    fn addi_zero_imm() {
        let (mut hart, mut bus) = setup();

        hart.set_xreg(1, 123);

        exec(encode_addi(3, 1, 0), &mut hart, &mut bus);

        assert_eq!(hart.xreg(3), 123);
    }

    #[test]
    fn addi_negative_imm() {
        let (mut hart, mut bus) = setup();

        hart.set_xreg(1, 5);

        exec(encode_addi(3, 1, -5), &mut hart, &mut bus);

        assert_eq!(hart.xreg(3), 0);
    }

    #[test]
    fn addi_overflow_wrap() {
        let (mut hart, mut bus) = setup();

        hart.set_xreg(1, u64::MAX);

        exec(encode_addi(3, 1, 1), &mut hart, &mut bus);

        assert_eq!(hart.xreg(3), 0);
    }

    #[test]
    fn addi_rd_rs1_same() {
        let (mut hart, mut bus) = setup();

        hart.set_xreg(1, 10);

        exec(encode_addi(1, 1, 5), &mut hart, &mut bus);

        assert_eq!(hart.xreg(1), 15);
    }

    #[test]
    fn addi_x0_source() {
        let (mut hart, mut bus) = setup();

        exec(encode_addi(3, 0, 7), &mut hart, &mut bus);

        assert_eq!(hart.xreg(3), 7);
    }

    #[test]
    fn addi_x0_destination() {
        let (mut hart, mut bus) = setup();

        hart.set_xreg(1, 5);

        exec(encode_addi(0, 1, 10), &mut hart, &mut bus);

        assert_eq!(hart.xreg(0), 0);
    }

    #[test]
    fn addi_max_positive_imm() {
        let (mut hart, mut bus) = setup();

        hart.set_xreg(1, 1);

        exec(encode_addi(3, 1, 2047), &mut hart, &mut bus);

        assert_eq!(hart.xreg(3), 2048);
    }

    #[test]
    fn addi_max_negative_imm() {
        let (mut hart, mut bus) = setup();

        hart.set_xreg(1, 0);

        exec(encode_addi(3, 1, -2048), &mut hart, &mut bus);

        assert_eq!(hart.xreg(3), (-2048i64) as u64);
    }

    #[test]
    fn addi_signed_boundary() {
        let (mut hart, mut bus) = setup();

        hart.set_xreg(1, 0x7fffffffffffffff);

        exec(encode_addi(3, 1, 1), &mut hart, &mut bus);

        assert_eq!(hart.xreg(3), 0x8000000000000000);
    }
}
