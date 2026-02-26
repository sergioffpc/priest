use crate::{
    memory::Bus,
    processor::riscv::{hart::Hart, instruction::InstrExec},
};

#[derive(Debug)]
pub struct Auipc;

impl InstrExec for Auipc {
    #[inline(always)]
    fn matches(&self, inst: u32) -> bool {
        inst & 0x7f == 0x17
    }

    #[inline(always)]
    #[cfg_attr(feature = "trace", tracing::instrument(name = "AUIPC", skip_all, fields(rd = tracing::field::Empty, imm = tracing::field::Empty)))]
    fn call(&self, inst: u32, hart: &mut Hart, _bus: &mut dyn Bus) -> anyhow::Result<()> {
        let rd = ((inst >> 7) & 0x1f) as usize;
        let imm = (inst & 0xfffff000) as u64;

        #[cfg(feature = "trace")]
        {
            let span = tracing::Span::current();
            span.record("rd", Hart::IABI[rd]);
            span.record("imm", imm);
        }

        hart.set_xreg(rd, hart.pc().wrapping_add(imm));
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

    fn encode_auipc(rd: u32, imm20: u32) -> u32 {
        ((imm20 & 0xfffff) << 12) | (rd << 7) | 0b0010111
    }

    fn setup() -> (Hart, Mmap) {
        (Hart::new(0), Mmap::new(0x0, 0x10_0000))
    }

    fn exec(inst: u32, hart: &mut Hart, bus: &mut Mmap) {
        Auipc
            .call(inst, hart, bus)
            .expect("AUIPC execution unexpectedly trapped");
    }

    #[test]
    fn auipc_basic() {
        let (mut hart, mut bus) = setup();
        hart.set_pc(0x1000);
        exec(encode_auipc(1, 0x1), &mut hart, &mut bus);
        assert_eq!(hart.xreg(1), 0x2000);
        assert_eq!(hart.pc(), 0x1004);
    }

    #[test]
    fn auipc_zero_imm() {
        let (mut hart, mut bus) = setup();
        hart.set_pc(0x5000);
        exec(encode_auipc(2, 0), &mut hart, &mut bus);
        assert_eq!(hart.xreg(2), 0x5000);
        assert_eq!(hart.pc(), 0x5004);
    }

    #[test]
    fn auipc_rd_x0() {
        let (mut hart, mut bus) = setup();
        hart.set_pc(0x1234);
        exec(encode_auipc(0, 0x1), &mut hart, &mut bus);
        assert_eq!(hart.xreg(0), 0);
        assert_eq!(hart.pc(), 0x1238);
    }

    #[test]
    fn auipc_large_imm() {
        let (mut hart, mut bus) = setup();
        hart.set_pc(0);
        exec(encode_auipc(3, 0xfffff), &mut hart, &mut bus);
        assert_eq!(hart.xreg(3), 0xfffff000);
        assert_eq!(hart.pc(), 0x4);
    }

    #[test]
    fn auipc_pc_nonzero_large_imm() {
        let (mut hart, mut bus) = setup();
        hart.set_pc(0x1000);
        exec(encode_auipc(4, 0xfffff), &mut hart, &mut bus);
        assert_eq!(hart.xreg(4), 0xfffff000 + 0x1000);
        assert_eq!(hart.pc(), 0x1004);
    }
}
