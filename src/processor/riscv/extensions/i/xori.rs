use crate::{
    memory::Bus,
    processor::riscv::{hart::Hart, instruction::InstrExec},
};

#[derive(Debug)]
pub struct Xori;

impl InstrExec for Xori {
    #[inline(always)]
    fn matches(&self, inst: u32) -> bool {
        inst & 0x707f == 0x4013
    }

    #[inline(always)]
    #[cfg_attr(feature = "trace", tracing::instrument(name = "XORI", skip_all, fields(rd = tracing::field::Empty, rs1 = tracing::field::Empty, imm = tracing::field::Empty)))]
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

        let val = hart.xreg(rs1) ^ imm;
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

    fn encode_xori(rd: usize, rs1: usize, imm: i16) -> u32 {
        let imm_u = imm as u32 & 0xfff;
        (imm_u << 20) | ((rs1 as u32) << 15) | (0b100 << 12) | ((rd as u32) << 7) | 0b0010011
    }

    fn exec(inst: u32, hart: &mut Hart, bus: &mut Mmap) {
        let instr = Xori;
        instr
            .call(inst, hart, bus)
            .expect("XORI execution unexpectedly trapped");
    }

    #[test]
    fn xori_basic() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 0b1010);
        exec(encode_xori(2, 1, 0b1100), &mut hart, &mut bus);
        assert_eq!(hart.xreg(2), 0b0110);
    }

    #[test]
    fn xori_zero() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 0);
        exec(encode_xori(2, 1, 0), &mut hart, &mut bus);
        assert_eq!(hart.xreg(2), 0);
    }

    #[test]
    fn xori_negative_imm() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 10);
        exec(encode_xori(2, 1, -1), &mut hart, &mut bus);
        assert_eq!(hart.xreg(2), 10 ^ 0xffff_ffff_ffff_ffff);
    }

    #[test]
    fn xori_large_values() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 0xffff_0000_ffff_0000);
        exec(encode_xori(2, 1, 0x0f0), &mut hart, &mut bus);
        assert_eq!(hart.xreg(2), 0xffff_0000_ffff_00f0);
    }

    #[test]
    fn xori_self() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 0b1111);
        exec(encode_xori(1, 1, 0b1111), &mut hart, &mut bus);
        assert_eq!(hart.xreg(1), 0);
    }
}
