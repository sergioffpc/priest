use std::{fmt::Debug, sync::LazyLock};

use crate::{
    memory::Bus,
    processor::riscv::{exception::Trap, extensions::i, hart::Hart},
};

pub static ISA: LazyLock<InstrTable> = LazyLock::new(InstrTable::default);

pub trait InstrExec: Debug + Send + Sync {
    fn matches(&self, inst: u32) -> bool;

    fn call(&self, inst: u32, hart: &mut Hart, bus: &mut dyn Bus) -> anyhow::Result<()>;
}

#[derive(Debug)]
pub struct InstrTable(Box<[Box<dyn InstrExec>]>);

impl InstrTable {
    pub fn dispatch(&self, inst: u32, hart: &mut Hart, bus: &mut dyn Bus) -> anyhow::Result<()> {
        for exec in self.0.iter() {
            if exec.matches(inst) {
                return exec.call(inst, hart, bus);
            }
        }
        Err(Trap::IllegalInstruction { inst }.into())
    }
}

impl Default for InstrTable {
    fn default() -> Self {
        let table: Vec<Box<dyn InstrExec>> = vec![
            Box::new(i::add::Add),
            Box::new(i::addi::Addi),
            Box::new(i::addiw::Addiw),
            Box::new(i::addw::Addw),
            Box::new(i::and::And),
            Box::new(i::andi::Andi),
            Box::new(i::auipc::Auipc),
            Box::new(i::beq::Beq),
            Box::new(i::bge::Bge),
            Box::new(i::bgeu::Bgeu),
            Box::new(i::blt::Blt),
            Box::new(i::bltu::Bltu),
            Box::new(i::bne::Bne),
            Box::new(i::ecall::Ebreak),
            Box::new(i::ebreak::Ecall),
            Box::new(i::fence::Fence),
            Box::new(i::jal::Jal),
            Box::new(i::jalr::Jalr),
            Box::new(i::lb::Lb),
            Box::new(i::lbu::Lbu),
            Box::new(i::ld::Ld),
            Box::new(i::lh::Lh),
            Box::new(i::lhu::Lhu),
            Box::new(i::lui::Lui),
            Box::new(i::lw::Lw),
            Box::new(i::lwu::Lwu),
            Box::new(i::or::Or),
            Box::new(i::ori::Ori),
            Box::new(i::sb::Sb),
            Box::new(i::sd::Sd),
            Box::new(i::sh::Sh),
            Box::new(i::sll::Sll),
            Box::new(i::slli::Slli),
            Box::new(i::slliw::Slliw),
            Box::new(i::sllw::Sllw),
            Box::new(i::slt::Slt),
            Box::new(i::slti::Slti),
            Box::new(i::sltiu::Sltiu),
            Box::new(i::sltu::Sltu),
            Box::new(i::sra::Sra),
            Box::new(i::srai::Srai),
            Box::new(i::sraiw::Sraiw),
            Box::new(i::sraw::Sraw),
            Box::new(i::srl::Srl),
            Box::new(i::srli::Srli),
            Box::new(i::srliw::Srliw),
            Box::new(i::srlw::Srlw),
            Box::new(i::sub::Sub),
            Box::new(i::subw::Subw),
            Box::new(i::sw::Sw),
            Box::new(i::xor::Xor),
            Box::new(i::xori::Xori),
        ];

        Self(table.into_boxed_slice())
    }
}
