use crate::{
    memory::Bus,
    processor::riscv::{hart::Hart, instruction::InstrExec},
};

#[derive(Debug)]
pub struct Sw;

impl InstrExec for Sw {
    #[inline(always)]
    fn matches(&self, inst: u32) -> bool {
        inst & 0x707f == 0x2023
    }

    #[inline(always)]
    #[cfg_attr(feature = "trace", tracing::instrument(name = "SW", skip_all, fields(rs1 = tracing::field::Empty, rs2 = tracing::field::Empty, imm = tracing::field::Empty)))]
    fn call(&self, inst: u32, hart: &mut Hart, bus: &mut dyn Bus) -> anyhow::Result<()> {
        let rs1 = ((inst >> 15) & 0x1f) as usize;
        let rs2 = ((inst >> 20) & 0x1f) as usize;
        let imm = ((((inst >> 7) & 0x1f) | (((inst >> 25) & 0x7f) << 5)) as i32) << 20 >> 20;

        #[cfg(feature = "trace")]
        {
            let span = tracing::Span::current();
            span.record("rs1", Hart::IABI[rs1]);
            span.record("rs2", Hart::IABI[rs2]);
            span.record("imm", imm);
        }

        let addr = hart.xreg(rs1).wrapping_add_signed(imm as i64);
        bus.write32(addr, hart.xreg(rs2) as u32)?;
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

    fn encode_sw(rs2: usize, rs1: usize, imm: i16) -> u32 {
        let imm_u = imm as u32;
        ((imm_u >> 5) << 25)
            | ((rs2 as u32) << 20)
            | ((rs1 as u32) << 15)
            | (0b010 << 12)
            | ((imm_u & 0x1f) << 7)
            | 0b0100011
    }

    fn exec(inst: u32, hart: &mut Hart, bus: &mut Mmap) {
        let instr = Sw;
        instr
            .call(inst, hart, bus)
            .expect("SW execution unexpectedly trapped");
    }

    #[test]
    fn sw_basic() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 0x1000);
        hart.set_xreg(2, 0xdeadbeef);
        exec(encode_sw(2, 1, 0), &mut hart, &mut bus);
        let val = bus.read32(0x1000).unwrap();
        assert_eq!(val, 0xdeadbeef);
    }

    #[test]
    fn sw_with_offset() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 0x2000);
        hart.set_xreg(2, 0x12345678);
        exec(encode_sw(2, 1, 16), &mut hart, &mut bus);
        let val = bus.read32(0x2010).unwrap();
        assert_eq!(val, 0x12345678);
    }

    #[test]
    fn sw_overwrite() {
        let (mut hart, mut bus) = setup();
        bus.write32(0x3000, 0x11111111).unwrap();
        hart.set_xreg(1, 0x3000);
        hart.set_xreg(2, 0x22222222);
        exec(encode_sw(2, 1, 0), &mut hart, &mut bus);
        let val = bus.read32(0x3000).unwrap();
        assert_eq!(val, 0x22222222);
    }

    #[test]
    fn sw_negative_offset() {
        let (mut hart, mut bus) = setup();
        hart.set_xreg(1, 0x4000);
        hart.set_xreg(2, 0xabcdef01);
        exec(encode_sw(2, 1, -4), &mut hart, &mut bus);
        let val = bus.read32(0x3ffc).unwrap();
        assert_eq!(val, 0xabcdef01);
    }
}
