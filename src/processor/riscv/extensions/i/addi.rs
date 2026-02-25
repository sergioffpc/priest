use crate::{
    memory::Bus,
    processor::riscv::{hart::Hart, instruction::InstrExec},
};

#[derive(Debug)]
pub struct Addi;

impl InstrExec for Addi {
    #[inline(always)]
    fn matches(&self, inst: u32) -> bool {
        inst & 0x707f == 0x13
    }

    #[inline(always)]
    #[cfg_attr(feature = "trace", tracing::instrument(name = "ADDI", skip_all, fields(rd = tracing::field::Empty, rs1 = tracing::field::Empty, imm = tracing::field::Empty)))]
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

        hart.set_xreg(rd, hart.xreg(rs1).wrapping_add_signed(imm));
        hart.next_pc();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyBus;

    impl Bus for DummyBus {
        fn fetch(&self, _paddr: u64) -> u32 {
            unreachable!()
        }

        fn read8(&self, _paddr: u64) -> anyhow::Result<u8> {
            unreachable!()
        }

        fn read16(&self, _paddr: u64) -> anyhow::Result<u16> {
            unreachable!()
        }

        fn read32(&self, _paddr: u64) -> anyhow::Result<u32> {
            unreachable!()
        }

        fn read64(&self, _paddr: u64) -> anyhow::Result<u64> {
            unreachable!()
        }

        fn write8(&mut self, _paddr: u64, _val: u8) -> anyhow::Result<()> {
            unreachable!()
        }

        fn write16(&mut self, _paddr: u64, _val: u16) -> anyhow::Result<()> {
            unreachable!()
        }

        fn write32(&mut self, _paddr: u64, _val: u32) -> anyhow::Result<()> {
            unreachable!()
        }

        fn write64(&mut self, _paddr: u64, _val: u64) -> anyhow::Result<()> {
            unreachable!()
        }
    }

    fn encode_addi(rd: u32, rs1: u32, imm: i32) -> u32 {
        let imm12 = (imm as u32) & 0xfff;
        (imm12 << 20) | (rs1 << 15) | (0b000 << 12) | (rd << 7) | 0x13
    }

    #[test]
    fn test_addi_basic() {
        let addi = Addi;
        let mut hart = Hart::default();
        let mut bus = DummyBus;

        hart.set_xreg(2, 10);

        let inst = encode_addi(1, 2, 5);

        addi.call(inst, &mut hart, &mut bus).unwrap();

        assert_eq!(hart.xreg(1), 15);
    }

    #[test]
    fn test_addi_negative_imm() {
        let addi = Addi;
        let mut hart = Hart::default();
        let mut bus = DummyBus;

        hart.set_xreg(2, 10);

        let inst = encode_addi(1, 2, -5);

        addi.call(inst, &mut hart, &mut bus).unwrap();

        assert_eq!(hart.xreg(1), 5);
    }

    #[test]
    fn test_addi_overflow_wrap() {
        let addi = Addi;
        let mut hart = Hart::default();
        let mut bus = DummyBus;

        hart.set_xreg(2, u64::MAX);

        let inst = encode_addi(1, 2, 1);

        addi.call(inst, &mut hart, &mut bus).unwrap();

        assert_eq!(hart.xreg(1), u64::MIN);
    }

    #[test]
    fn test_addi_rd_x0_is_ignored() {
        let addi = Addi;
        let mut hart = Hart::default();
        let mut bus = DummyBus;

        hart.set_xreg(2, 123);

        let inst = encode_addi(0, 2, 5);

        addi.call(inst, &mut hart, &mut bus).unwrap();

        assert_eq!(hart.xreg(0), 0);
    }

    #[test]
    fn test_pc_increment() {
        let addi = Addi;
        let mut hart = Hart::default();
        let mut bus = DummyBus;

        let pc_before = hart.pc();

        let inst = encode_addi(1, 0, 1);

        addi.call(inst, &mut hart, &mut bus).unwrap();

        assert_eq!(hart.pc(), pc_before + 4);
    }

    #[test]
    fn test_addi_sign_extension_max_negative() {
        let addi = Addi;
        let mut hart = Hart::default();
        let mut bus = DummyBus;

        hart.set_xreg(2, 0);

        let inst = encode_addi(1, 2, -2048);

        addi.call(inst, &mut hart, &mut bus).unwrap();

        let val = -2048i64;
        assert_eq!(hart.xreg(1), val as u64);
    }
}
