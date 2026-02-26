use crate::{
    memory::Bus,
    processor::riscv::{hart::Hart, instruction::InstrExec},
};

#[derive(Debug)]
pub struct Bge;

impl InstrExec for Bge {
    #[inline(always)]
    fn matches(&self, inst: u32) -> bool {
        inst & 0x707f == 0x5063
    }

    #[inline(always)]
    #[cfg_attr(feature = "trace", tracing::instrument(name = "BGE", skip_all, fields(rs1 = tracing::field::Empty, rs2 = tracing::field::Empty, imm = tracing::field::Empty)))]
    fn call(&self, inst: u32, hart: &mut Hart, _bus: &mut dyn Bus) -> anyhow::Result<()> {
        let rs1 = ((inst >> 15) & 0x1f) as usize;
        let rs2 = ((inst >> 20) & 0x1f) as usize;
        let imm = {
            let mut val = ((inst >> 31) & 0x1) << 12;
            val |= ((inst >> 7) & 0x1) << 11;
            val |= ((inst >> 25) & 0x3f) << 5;
            val |= ((inst >> 8) & 0xf) << 1;
            ((val as i16) << 3) as i64 >> 3
        };

        #[cfg(feature = "trace")]
        {
            let span = tracing::Span::current();
            span.record("rs1", Hart::IABI[rs1]);
            span.record("rs2", Hart::IABI[rs2]);
            span.record("imm", imm);
        }

        if (hart.xreg(rs1) as i64) >= (hart.xreg(rs2) as i64) {
            let target = hart.pc().wrapping_add_signed(imm);
            if target & 0b11 != 0 {
                return Err(
                    crate::memory::exception::Trap::MisalignedFetch { addr: target }.into(),
                );
            }
            hart.set_pc(target);
        } else {
            hart.next_pc();
        }

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

    fn encode_bge(rs1: u32, rs2: u32, imm: i16) -> u32 {
        let imm12 = imm as u32;
        let imm_12 = ((imm12 >> 12) & 0x1) << 31;
        let imm_10_5 = ((imm12 >> 5) & 0x3f) << 25;
        let imm_4_1 = ((imm12 >> 1) & 0xf) << 8;
        let imm_11 = ((imm12 >> 11) & 0x1) << 7;
        imm_12 | imm_10_5 | (rs2 << 20) | (rs1 << 15) | (0b101 << 12) | imm_4_1 | imm_11 | 0b1100011
    }

    fn setup() -> (Hart, Mmap) {
        (Hart::new(0), Mmap::new(0x0, 0x10_0000))
    }

    fn exec(inst: u32, hart: &mut Hart, bus: &mut Mmap) {
        Bge.call(inst, hart, bus)
            .expect("BGE execution unexpectedly trapped");
    }

    #[test]
    fn bge_taken() {
        let (mut hart, mut bus) = setup();
        hart.set_pc(0x1000);
        hart.set_xreg(1, 10);
        hart.set_xreg(2, 5);
        exec(encode_bge(1, 2, 16), &mut hart, &mut bus);
        assert_eq!(hart.pc(), 0x1000 + 16);
    }

    #[test]
    fn bge_not_taken() {
        let (mut hart, mut bus) = setup();
        hart.set_pc(0x2000);
        hart.set_xreg(1, 3);
        hart.set_xreg(2, 5);
        exec(encode_bge(1, 2, 16), &mut hart, &mut bus);
        assert_eq!(hart.pc(), 0x2004);
    }

    #[test]
    fn bge_equal() {
        let (mut hart, mut bus) = setup();
        hart.set_pc(0x3000);
        hart.set_xreg(1, 7);
        hart.set_xreg(2, 7);
        exec(encode_bge(1, 2, 12), &mut hart, &mut bus);
        assert_eq!(hart.pc(), 0x3000 + 12);
    }

    #[test]
    fn bge_negative_offset() {
        let (mut hart, mut bus) = setup();
        hart.set_pc(0x4000);
        hart.set_xreg(1, 5);
        hart.set_xreg(2, 2);
        exec(encode_bge(1, 2, -8), &mut hart, &mut bus);
        assert_eq!(hart.pc(), 0x4000 - 8);
    }

    #[test]
    fn bge_zero_offset() {
        let (mut hart, mut bus) = setup();
        hart.set_pc(0x5000);
        hart.set_xreg(1, 3);
        hart.set_xreg(2, 3);
        exec(encode_bge(1, 2, 0), &mut hart, &mut bus);
        assert_eq!(hart.pc(), 0x5000);
    }
}
