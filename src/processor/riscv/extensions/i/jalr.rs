use crate::{
    memory::Bus,
    processor::riscv::{hart::Hart, instruction::InstrExec},
};

#[derive(Debug)]
pub struct Jalr;

impl InstrExec for Jalr {
    #[inline(always)]
    fn matches(&self, inst: u32) -> bool {
        inst & 0x707f == 0x67
    }

    #[inline(always)]
    #[cfg_attr(feature = "trace", tracing::instrument(name = "JALR", skip_all, fields(rd = tracing::field::Empty, rs1 = tracing::field::Empty, imm = tracing::field::Empty)))]
    fn call(&self, inst: u32, hart: &mut Hart, _bus: &mut dyn Bus) -> anyhow::Result<()> {
        let rd = ((inst >> 7) & 0x1f) as usize;
        let rs1 = ((inst >> 15) & 0x1f) as usize;
        let imm = ((inst as i32) >> 20) as i64;

        #[cfg(feature = "trace")]
        {
            let span = tracing::Span::current();
            span.record("rd", Hart::IABI[rd]);
            span.record("rs1", Hart::IABI[rs1]);
            span.record("imm", imm);
        }

        let target = hart.xreg(rs1).wrapping_add_signed(imm) & !1;
        if target & 0b11 != 0 {
            return Err(crate::memory::exception::Trap::MisalignedFetch { addr: target }.into());
        }

        let pc = hart.pc();
        hart.set_xreg(rd, pc.wrapping_add(Hart::ILEN));

        hart.set_pc(target);

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

    fn encode_jalr(rd: u32, rs1: u32, imm: i16) -> u32 {
        let imm12 = imm as u32 & 0xfff;
        (imm12 << 20) | (rs1 << 15) | (0b000 << 12) | (rd << 7) | 0b1100111
    }

    fn setup() -> (Hart, Mmap) {
        (Hart::new(0), Mmap::new(0x0, 0x10_0000))
    }

    fn exec(inst: u32, hart: &mut Hart, bus: &mut Mmap) {
        Jalr.call(inst, hart, bus)
            .expect("JALR execution unexpectedly trapped");
    }

    #[test]
    fn jalr_basic() {
        let (mut hart, mut bus) = setup();
        hart.set_pc(0x1000);
        hart.set_xreg(1, 0x2000);
        exec(encode_jalr(2, 1, 16), &mut hart, &mut bus);
        assert_eq!(hart.xreg(2), 0x1000 + 4);
        assert_eq!(hart.pc(), 0x2000 + 16);
    }

    #[test]
    fn jalr_zero_offset() {
        let (mut hart, mut bus) = setup();
        hart.set_pc(0x3000);
        hart.set_xreg(1, 0x4000);
        exec(encode_jalr(2, 1, 0), &mut hart, &mut bus);
        assert_eq!(hart.xreg(2), 0x3000 + 4);
        assert_eq!(hart.pc(), 0x4000);
    }

    #[test]
    fn jalr_rd_x0() {
        let (mut hart, mut bus) = setup();
        hart.set_pc(0x5000);
        hart.set_xreg(1, 0x6000);
        exec(encode_jalr(0, 1, 8), &mut hart, &mut bus);
        assert_eq!(hart.xreg(0), 0);
        assert_eq!(hart.pc(), 0x6000 + 8);
    }

    #[test]
    fn jalr_negative_offset() {
        let (mut hart, mut bus) = setup();
        hart.set_pc(0x7000);
        hart.set_xreg(1, 0x8000);
        exec(encode_jalr(3, 1, -12), &mut hart, &mut bus);
        assert_eq!(hart.xreg(3), 0x7000 + 4);
        assert_eq!(hart.pc(), 0x8000 - 12);
    }

    #[test]
    fn jalr_rd_rs1_same() {
        let (mut hart, mut bus) = setup();
        hart.set_pc(0x9000);
        hart.set_xreg(1, 0x1000);
        exec(encode_jalr(1, 1, 20), &mut hart, &mut bus);
        assert_eq!(hart.xreg(1), 0x9000 + 4);
        assert_eq!(hart.pc(), 0x1000 + 20);
    }
}
