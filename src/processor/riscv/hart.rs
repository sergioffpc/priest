use crate::{
    memory::Bus,
    processor::{Cpu, riscv::instruction::ISA},
};

#[derive(Debug, Default)]
pub struct Hart {
    pc: u64,
    xregs: [u64; 32],
}

impl Hart {
    pub const ILEN: u64 = 4;
    pub const IABI: [&str; 32] = [
        "zero", "ra", "sp", "gp", "tp", "t0", "t1", "t2", "s0", "s1", "a0", "a1", "a2", "a3", "a4",
        "a5", "a6", "a7", "s2", "s3", "s4", "s5", "s6", "s7", "s8", "s9", "s10", "s11", "t3", "t4",
        "t5", "t6",
    ];

    pub fn new(entry: u64) -> Self {
        Self {
            pc: entry,
            xregs: [0u64; 32],
        }
    }

    #[inline(always)]
    pub fn pc(&self) -> u64 {
        self.pc
    }

    #[inline(always)]
    pub fn set_pc(&mut self, paddr: u64) {
        self.pc = paddr;
    }

    #[inline(always)]
    pub fn next_pc(&mut self) {
        self.pc = self.pc.wrapping_add(Self::ILEN);
    }

    #[inline(always)]
    pub fn xreg(&self, i: usize) -> u64 {
        unsafe { *self.xregs.get_unchecked(i) }
    }

    #[inline(always)]
    pub fn set_xreg(&mut self, i: usize, val: u64) {
        if i != 0 {
            unsafe {
                *self.xregs.get_unchecked_mut(i) = val;
            }
        }
    }
}

impl std::fmt::Display for Hart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "pc 0x{:018x}", self.pc)?;
        for row in 0..8 {
            let i = row * 4;
            writeln!(
                f,
                "x{:<2} [{:<4}] 0x{:018x}    x{:<2} [{:<4}] 0x{:018x}    x{:<2} [{:<4}] 0x{:018x}    x{:<2} [{:<4}] 0x{:018x}",
                i,
                Self::IABI[i],
                self.xregs[i],
                i + 1,
                Self::IABI[i + 1],
                self.xregs[i + 1],
                i + 2,
                Self::IABI[i + 2],
                self.xregs[i + 2],
                i + 3,
                Self::IABI[i + 3],
                self.xregs[i + 3],
            )?;
        }
        Ok(())
    }
}

impl Cpu for Hart {
    #[inline(always)]
    fn step<B>(&mut self, bus: &mut B) -> anyhow::Result<()>
    where
        B: Bus,
    {
        let inst = bus.fetch(self.pc);
        ISA.dispatch(inst, self, bus)
    }
}
