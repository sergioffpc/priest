use crate::{
    memory::Bus,
    processor::riscv::{hart::Hart, instruction::InstrExec},
};

#[derive(Debug)]
pub struct Ori;

impl InstrExec for Ori {
    #[inline(always)]
    fn matches(&self, inst: u32) -> bool {
        inst & 0x707f == 0x6013
    }

    #[inline(always)]
    #[cfg_attr(feature = "trace", tracing::instrument(name = "ORI", skip_all, fields(rd = tracing::field::Empty, rs1 = tracing::field::Empty, imm = tracing::field::Empty)))]
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

        let val = hart.xreg(rs1) | imm;
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

    fn encode_ori(rd: u32, rs1: u32, imm: i16) -> u32 {
        let imm12 = (imm as u32) & 0xfff;
        (imm12 << 20) | (rs1 << 15) | (0b110 << 12) | (rd << 7) | 0b0010011
    }

    fn setup() -> (Hart, Mmap) {
        (Hart::new(0), Mmap::new(0x0, 0x10_0000))
    }

    fn exec(inst: u32, hart: &mut Hart, bus: &mut Mmap) {
        Ori.call(inst, hart, bus)
            .expect("ORI execution unexpectedly trapped");
    }

    #[test]
    fn ori_basic() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 0x1234);

        exec(encode_ori(2, 1, 0x00ff), &mut hart, &mut bus);
        assert_eq!(hart.xreg(2), 0x12ff);
    }

    #[test]
    fn ori_zero() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 0);

        exec(encode_ori(2, 1, 0), &mut hart, &mut bus);
        assert_eq!(hart.xreg(2), 0);
    }

    #[test]
    fn ori_negative_imm() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 0x10);

        exec(encode_ori(2, 1, -1), &mut hart, &mut bus);
        assert_eq!(hart.xreg(2), 0xffffffffffffffff);
    }

    #[test]
    fn ori_rd_x0() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 0x1234);

        exec(encode_ori(0, 1, 0x567), &mut hart, &mut bus);
        assert_eq!(hart.xreg(0), 0);
    }

    #[test]
    fn ori_x0_source() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 0x4321);

        exec(encode_ori(2, 1, 0), &mut hart, &mut bus);
        assert_eq!(hart.xreg(2), 0x4321);
    }
}
