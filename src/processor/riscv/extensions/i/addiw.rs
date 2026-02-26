use crate::{
    memory::Bus,
    processor::riscv::{hart::Hart, instruction::InstrExec},
};

#[derive(Debug)]
pub struct Addiw;

impl InstrExec for Addiw {
    #[inline(always)]
    fn matches(&self, inst: u32) -> bool {
        inst & 0x707f == 0x1b
    }

    #[inline(always)]
    #[cfg_attr(feature = "trace", tracing::instrument(name = "ADDIW", skip_all, fields(rd = tracing::field::Empty, rs1 = tracing::field::Empty, imm = tracing::field::Empty)))]
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

        let val = hart.xreg(rs1).wrapping_add(imm);
        let val = val as i32 as i64;

        hart.set_xreg(rd, val as u64);
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

    fn encode_addiw(rd: u32, rs1: u32, imm: i32) -> u32 {
        let imm12 = (imm as u32) & 0xfff;
        (imm12 << 20) | (rs1 << 15) | (0b000 << 12) | (rd << 7) | 0b0011011
    }

    fn setup() -> (Hart, Mmap) {
        (Hart::new(0), Mmap::new(0x0, 0x10_0000))
    }

    fn exec(inst: u32, hart: &mut Hart, bus: &mut Mmap) {
        Addiw
            .call(inst, hart, bus)
            .expect("ADDIW execution unexpectedly trapped");
    }

    #[test]
    fn addiw_basic() {
        let (mut hart, mut bus) = setup();

        hart.set_xreg(1, 1);

        exec(encode_addiw(3, 1, 2), &mut hart, &mut bus);

        assert_eq!(hart.xreg(3), 3);
    }

    #[test]
    fn addiw_sign_extend_positive() {
        let (mut hart, mut bus) = setup();

        hart.set_xreg(1, 0x000000007fffffff);

        exec(encode_addiw(3, 1, 0), &mut hart, &mut bus);

        assert_eq!(hart.xreg(3), 0x7fffffff);
    }

    #[test]
    fn addiw_sign_extend_negative() {
        let (mut hart, mut bus) = setup();

        hart.set_xreg(1, 0x0000000080000000);

        exec(encode_addiw(3, 1, 0), &mut hart, &mut bus);

        assert_eq!(hart.xreg(3), 0xffffffff80000000);
    }

    #[test]
    fn addiw_overflow_wrap_32bit() {
        let (mut hart, mut bus) = setup();

        hart.set_xreg(1, 0x7fffffff);

        exec(encode_addiw(3, 1, 1), &mut hart, &mut bus);

        assert_eq!(hart.xreg(3), 0xffffffff80000000);
    }

    #[test]
    fn addiw_negative_imm() {
        let (mut hart, mut bus) = setup();

        hart.set_xreg(1, 5);

        exec(encode_addiw(3, 1, -10), &mut hart, &mut bus);

        assert_eq!(hart.xreg(3), (-5i32 as i64) as u64);
    }

    #[test]
    fn addiw_rd_rs1_same() {
        let (mut hart, mut bus) = setup();

        hart.set_xreg(1, 100);

        exec(encode_addiw(1, 1, 23), &mut hart, &mut bus);

        assert_eq!(hart.xreg(1), 123);
    }

    #[test]
    fn addiw_x0_source() {
        let (mut hart, mut bus) = setup();

        exec(encode_addiw(3, 0, -1), &mut hart, &mut bus);

        assert_eq!(hart.xreg(3), 0xffffffffffffffff);
    }

    #[test]
    fn addiw_x0_destination() {
        let (mut hart, mut bus) = setup();

        hart.set_xreg(1, 123);

        exec(encode_addiw(0, 1, 456), &mut hart, &mut bus);

        assert_eq!(hart.xreg(0), 0);
    }

    #[test]
    fn addiw_ignores_upper_32bits_of_rs1() {
        let (mut hart, mut bus) = setup();

        hart.set_xreg(1, 0xffffffff00000001);

        exec(encode_addiw(3, 1, 1), &mut hart, &mut bus);

        assert_eq!(hart.xreg(3), 2);
    }
}
