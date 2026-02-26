use crate::{
    memory::Bus,
    processor::riscv::{hart::Hart, instruction::InstrExec},
};

#[derive(Debug)]
pub struct Andi;

impl InstrExec for Andi {
    #[inline(always)]
    fn matches(&self, inst: u32) -> bool {
        inst & 0x707f == 0x7013
    }

    #[inline(always)]
    #[cfg_attr(feature = "trace", tracing::instrument(name = "ANDI", skip_all, fields(rd = tracing::field::Empty, rs1 = tracing::field::Empty, imm = tracing::field::Empty)))]
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

        hart.set_xreg(rd, hart.xreg(rs1) & imm);
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

    fn encode_andi(rd: u32, rs1: u32, imm: i16) -> u32 {
        let imm12 = (imm as u32) & 0xfff;
        (imm12 << 20) | (rs1 << 15) | (0b111 << 12) | (rd << 7) | 0b0010011
    }

    fn setup() -> (Hart, Mmap) {
        (Hart::new(0), Mmap::new(0x0, 0x10_0000))
    }

    fn exec(inst: u32, hart: &mut Hart, bus: &mut Mmap) {
        Andi.call(inst, hart, bus)
            .expect("ANDI execution unexpectedly trapped");
    }

    #[test]
    fn andi_basic() {
        let (mut hart, mut bus) = setup();

        hart.set_xreg(1, 0b1100);

        exec(encode_andi(3, 1, 0b1010), &mut hart, &mut bus);

        assert_eq!(hart.xreg(3), 0b1000);
    }

    #[test]
    fn andi_zero_imm() {
        let (mut hart, mut bus) = setup();

        hart.set_xreg(1, 0b1100);

        exec(encode_andi(3, 1, 0), &mut hart, &mut bus);

        assert_eq!(hart.xreg(3), 0);
    }

    #[test]
    fn andi_zero_rs1() {
        let (mut hart, mut bus) = setup();

        exec(encode_andi(3, 0, 0b1010), &mut hart, &mut bus);

        assert_eq!(hart.xreg(3), 0);
    }

    #[test]
    fn andi_all_ones_12bit() {
        let (mut hart, mut bus) = setup();

        hart.set_xreg(1, u64::MAX);

        exec(encode_andi(3, 1, 0xfff), &mut hart, &mut bus); // 12-bit all ones

        assert_eq!(hart.xreg(3), u64::MAX);
    }

    #[test]
    fn andi_negative_imm() {
        let (mut hart, mut bus) = setup();

        hart.set_xreg(1, 0b1111);

        exec(encode_andi(3, 1, -5), &mut hart, &mut bus);

        // 0b1111 & (-5 sign-extended 12-bit) = 0b1111 & 0b111111111011 = 0b1011
        assert_eq!(hart.xreg(3), 0b1011);
    }

    #[test]
    fn andi_rd_rs1_same() {
        let (mut hart, mut bus) = setup();

        hart.set_xreg(1, 0b1111);

        exec(encode_andi(1, 1, 0b1010), &mut hart, &mut bus);

        assert_eq!(hart.xreg(1), 0b1010);
    }

    #[test]
    fn andi_x0_source() {
        let (mut hart, mut bus) = setup();

        exec(encode_andi(3, 0, 0b1111), &mut hart, &mut bus);

        assert_eq!(hart.xreg(3), 0);
    }

    #[test]
    fn andi_x0_destination() {
        let (mut hart, mut bus) = setup();

        hart.set_xreg(1, 0xfff);

        exec(encode_andi(0, 1, 0b1010), &mut hart, &mut bus);

        assert_eq!(hart.xreg(0), 0);
    }

    #[test]
    fn andi_mask_high_bits_12bit() {
        let (mut hart, mut bus) = setup();

        hart.set_xreg(1, 0xffffffffffffffff);

        exec(encode_andi(3, 1, 0x0f0), &mut hart, &mut bus); // ≤ 12-bit immediate

        assert_eq!(hart.xreg(3), 0x0f0);
    }
}
