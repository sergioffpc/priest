use crate::{
    memory::Bus,
    processor::riscv::{hart::Hart, instruction::InstrExec},
};

#[derive(Debug)]
pub struct Slti;

impl InstrExec for Slti {
    #[inline(always)]
    fn matches(&self, inst: u32) -> bool {
        inst & 0x707f == 0x2013
    }

    #[inline(always)]
    #[cfg_attr(feature = "trace", tracing::instrument(name = "SLTI", skip_all, fields(rd = tracing::field::Empty, rs1 = tracing::field::Empty, imm = tracing::field::Empty)))]
    fn call(&self, inst: u32, hart: &mut Hart, _bus: &mut dyn Bus) -> anyhow::Result<()> {
        let rd = ((inst >> 7) & 0x1f) as usize;
        let rs1 = ((inst >> 15) & 0x1f) as usize;

        // Sign-extend 12-bit immediate
        let imm = ((inst as i32) >> 20) as i64;

        #[cfg(feature = "trace")]
        {
            let span = tracing::Span::current();
            span.record("rd", Hart::IABI[rd]);
            span.record("rs1", Hart::IABI[rs1]);
            span.record("imm", imm);
        }

        let val = if (hart.xreg(rs1) as i64) < imm { 1 } else { 0 };
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

    fn encode_slti(rd: usize, rs1: usize, imm: i32) -> u32 {
        (((imm as u32) & 0xfff) << 20)
            | ((rs1 as u32) << 15)
            | (0b010 << 12)
            | ((rd as u32) << 7)
            | 0b0010011
    }

    fn exec(inst: u32, hart: &mut Hart, bus: &mut Mmap) {
        let instr = Slti;
        instr
            .call(inst, hart, bus)
            .expect("SLTI execution unexpectedly trapped");
    }

    #[test]
    fn slti_true() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 5);
        exec(encode_slti(2, 1, 10), &mut hart, &mut bus);
        assert_eq!(hart.xreg(2), 1);
    }

    #[test]
    fn slti_false() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 10);
        exec(encode_slti(2, 1, 5), &mut hart, &mut bus);
        assert_eq!(hart.xreg(2), 0);
    }

    #[test]
    fn slti_equal() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 7);
        exec(encode_slti(2, 1, 7), &mut hart, &mut bus);
        assert_eq!(hart.xreg(2), 0);
    }

    #[test]
    fn slti_negative_true() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, (-5i64) as u64);
        exec(encode_slti(2, 1, -1), &mut hart, &mut bus);
        assert_eq!(hart.xreg(2), 1);
    }

    #[test]
    fn slti_negative_false() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 1);
        exec(encode_slti(2, 1, -5), &mut hart, &mut bus);
        assert_eq!(hart.xreg(2), 0);
    }

    #[test]
    fn slti_self() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, (-10i64) as u64);
        exec(encode_slti(1, 1, -10), &mut hart, &mut bus);
        assert_eq!(hart.xreg(1), 0);
    }

    #[test]
    fn slti_max_negative_imm() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 0);
        exec(encode_slti(2, 1, -2048), &mut hart, &mut bus);
        assert_eq!(hart.xreg(2), 0);
    }

    #[test]
    fn slti_max_positive_imm() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 0);
        exec(encode_slti(2, 1, 2047), &mut hart, &mut bus);
        assert_eq!(hart.xreg(2), 1);
    }
}
