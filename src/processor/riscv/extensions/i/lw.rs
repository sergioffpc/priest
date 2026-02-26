use crate::{
    memory::Bus,
    processor::riscv::{hart::Hart, instruction::InstrExec},
};

#[derive(Debug)]
pub struct Lw;

impl InstrExec for Lw {
    #[inline(always)]
    fn matches(&self, inst: u32) -> bool {
        inst & 0x707f == 0x2003
    }

    #[inline(always)]
    #[cfg_attr(feature = "trace", tracing::instrument(name = "LW", skip_all, fields(rd = tracing::field::Empty, rs1 = tracing::field::Empty, imm = tracing::field::Empty)))]
    fn call(&self, inst: u32, hart: &mut Hart, bus: &mut dyn Bus) -> anyhow::Result<()> {
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

        let addr = hart.xreg(rs1).wrapping_add_signed(imm);
        let val = (bus.read32(addr)? as i32 as i64) as u64;

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

    fn encode_lw(rd: u32, rs1: u32, imm: i16) -> u32 {
        let imm12 = (imm as u32) & 0xfff;
        (imm12 << 20) | (rs1 << 15) | (0b010 << 12) | (rd << 7) | 0b0000011
    }

    fn setup() -> (Hart, Mmap) {
        (Hart::new(0), Mmap::new(0x0, 0x10_0000))
    }

    fn exec(inst: u32, hart: &mut Hart, bus: &mut Mmap) {
        Lw.call(inst, hart, bus)
            .expect("LW execution unexpectedly trapped");
    }

    #[test]
    fn lw_basic() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 0x100);
        bus.write32(0x100, 0x12345678).unwrap();

        exec(encode_lw(2, 1, 0), &mut hart, &mut bus);
        assert_eq!(hart.xreg(2), 0x12345678);
    }

    #[test]
    fn lw_with_offset() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 0x200);
        bus.write32(0x204, 0xdeadbeef).unwrap();

        exec(encode_lw(2, 1, 4), &mut hart, &mut bus);
        assert_eq!(hart.xreg(2), 0xffffffffdeadbeef);
    }

    #[test]
    fn lw_rd_x0() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 0x300);
        bus.write32(0x300, 0xabcdef12).unwrap();

        exec(encode_lw(0, 1, 0), &mut hart, &mut bus);
        assert_eq!(hart.xreg(0), 0);
    }

    #[test]
    fn lw_negative_offset() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 0x400);
        bus.write32(0x3fc, 0x11223344).unwrap();

        exec(encode_lw(2, 1, -4), &mut hart, &mut bus);
        assert_eq!(hart.xreg(2), 0x11223344);
    }
}
