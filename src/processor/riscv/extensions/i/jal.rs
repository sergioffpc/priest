use crate::{
    memory::Bus,
    processor::riscv::{hart::Hart, instruction::InstrExec},
};

#[derive(Debug)]
pub struct Jal;

impl InstrExec for Jal {
    #[inline(always)]
    fn matches(&self, inst: u32) -> bool {
        inst & 0x7f == 0x6f
    }

    #[inline(always)]
    #[cfg_attr(feature = "trace", tracing::instrument(name = "JAL", skip_all, fields(rd = tracing::field::Empty, imm = tracing::field::Empty)))]
    fn call(&self, inst: u32, hart: &mut Hart, _bus: &mut dyn Bus) -> anyhow::Result<()> {
        let rd = ((inst >> 7) & 0x1f) as usize;

        let mut imm = ((inst >> 31) & 0x1) << 20;
        imm |= ((inst >> 12) & 0xff) << 12;
        imm |= ((inst >> 20) & 0x1) << 11;
        imm |= ((inst >> 21) & 0x3ff) << 1;
        let imm = ((imm as i32) << 11) as i64 >> 11;

        #[cfg(feature = "trace")]
        {
            let span = tracing::Span::current();
            span.record("rd", Hart::IABI[rd]);
            span.record("imm", imm);
        }

        let pc = hart.pc();
        hart.set_xreg(rd, pc.wrapping_add(Hart::ILEN));

        let target = pc.wrapping_add_signed(imm);
        if target & 0b11 != 0 {
            return Err(crate::memory::exception::Trap::MisalignedFetch { addr: target }.into());
        }
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

    fn encode_jal(rd: u32, imm: i32) -> u32 {
        let imm_u = imm as u32;
        let imm_20 = ((imm_u >> 20) & 0x1) << 31;
        let imm_10_1 = ((imm_u >> 1) & 0x3ff) << 21;
        let imm_11 = ((imm_u >> 11) & 0x1) << 20;
        let imm_19_12 = ((imm_u >> 12) & 0xff) << 12;
        imm_20 | imm_19_12 | imm_11 | imm_10_1 | (rd << 7) | 0b1101111
    }

    fn setup() -> (Hart, Mmap) {
        (Hart::new(0), Mmap::new(0x0, 0x10_0000))
    }

    fn exec(inst: u32, hart: &mut Hart, bus: &mut Mmap) {
        Jal.call(inst, hart, bus)
            .expect("JAL execution unexpectedly trapped");
    }

    #[test]
    fn jal_basic() {
        let (mut hart, mut bus) = setup();
        hart.set_pc(0x1000);
        exec(encode_jal(1, 16), &mut hart, &mut bus);
        assert_eq!(hart.xreg(1), 0x1000 + 4);
        assert_eq!(hart.pc(), 0x1000 + 16);
    }

    #[test]
    fn jal_zero_offset() {
        let (mut hart, mut bus) = setup();
        hart.set_pc(0x2000);
        exec(encode_jal(2, 0), &mut hart, &mut bus);
        assert_eq!(hart.xreg(2), 0x2000 + 4);
        assert_eq!(hart.pc(), 0x2000);
    }

    #[test]
    fn jal_rd_x0() {
        let (mut hart, mut bus) = setup();
        hart.set_pc(0x3000);
        exec(encode_jal(0, 8), &mut hart, &mut bus);
        assert_eq!(hart.xreg(0), 0);
        assert_eq!(hart.pc(), 0x3000 + 8);
    }

    #[test]
    fn jal_negative_offset() {
        let (mut hart, mut bus) = setup();
        hart.set_pc(0x4000);
        exec(encode_jal(3, -12), &mut hart, &mut bus);
        assert_eq!(hart.xreg(3), 0x4000 + 4);
        assert_eq!(hart.pc(), 0x4000 - 12);
    }

    #[test]
    fn jal_large_offset() {
        let (mut hart, mut bus) = setup();
        hart.set_pc(0x1000);
        exec(encode_jal(4, 0xffff0), &mut hart, &mut bus);
        assert_eq!(hart.xreg(4), 0x1000 + 4);
        assert_eq!(hart.pc(), 0x1000 + 0xffff0);
    }
}
