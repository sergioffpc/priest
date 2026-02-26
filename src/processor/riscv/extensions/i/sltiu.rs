use crate::{
    memory::Bus,
    processor::riscv::{hart::Hart, instruction::InstrExec},
};

#[derive(Debug)]
pub struct Sltiu;

impl InstrExec for Sltiu {
    #[inline(always)]
    fn matches(&self, inst: u32) -> bool {
        inst & 0x707f == 0x3013
    }

    #[inline(always)]
    #[cfg_attr(feature = "trace", tracing::instrument(name = "SLTIU", skip_all, fields(rd = tracing::field::Empty, rs1 = tracing::field::Empty, imm = tracing::field::Empty)))]
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

        let val = if hart.xreg(rs1) < imm { 1 } else { 0 };
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

    fn encode_sltiu(rd: usize, rs1: usize, imm: i32) -> u32 {
        (((imm as u32) & 0xfff) << 20)
            | ((rs1 as u32) << 15)
            | (0b011 << 12)
            | ((rd as u32) << 7)
            | 0b0010011
    }

    fn exec(inst: u32, hart: &mut Hart, bus: &mut Mmap) {
        let instr = Sltiu;
        instr
            .call(inst, hart, bus)
            .expect("SLTIU execution unexpectedly trapped");
    }

    #[test]
    fn sltiu_true() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 5);
        exec(encode_sltiu(2, 1, 10), &mut hart, &mut bus);
        assert_eq!(hart.xreg(2), 1);
    }

    #[test]
    fn sltiu_false() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 10);
        exec(encode_sltiu(2, 1, 5), &mut hart, &mut bus);
        assert_eq!(hart.xreg(2), 0);
    }

    #[test]
    fn sltiu_equal() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 7);
        exec(encode_sltiu(2, 1, 7), &mut hart, &mut bus);
        assert_eq!(hart.xreg(2), 0);
    }

    #[test]
    fn sltiu_negative_imm() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 0);
        exec(encode_sltiu(2, 1, -1), &mut hart, &mut bus);
        assert_eq!(hart.xreg(2), 1);
    }

    #[test]
    fn sltiu_rs1_max() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, u64::MAX);
        exec(encode_sltiu(2, 1, -1), &mut hart, &mut bus);
        assert_eq!(hart.xreg(2), 0);
    }

    #[test]
    fn sltiu_self() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 123);
        exec(encode_sltiu(1, 1, 124), &mut hart, &mut bus);
        assert_eq!(hart.xreg(1), 1);
    }

    #[test]
    fn sltiu_max_positive_imm() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 2046);
        exec(encode_sltiu(2, 1, 2047), &mut hart, &mut bus);
        assert_eq!(hart.xreg(2), 1);
    }

    #[test]
    fn sltiu_max_negative_imm() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 0);
        exec(encode_sltiu(2, 1, -2048), &mut hart, &mut bus);
        assert_eq!(hart.xreg(2), 1);
    }
}
