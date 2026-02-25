use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
pub enum Trap {
    #[error("instruction address misaligned at 0x{addr:016x}")]
    MisalignedFetch { addr: u64 },

    #[error("instruction access fault at 0x{addr:016x}")]
    FetchAccessFault { addr: u64 },

    #[error("load access fault at address 0x{addr:016x}")]
    LoadAccessFault { addr: u64 },

    #[error("store access fault at address 0x{addr:016x}")]
    StoreAccessFault { addr: u64 },

    #[error("load address misaligned at address 0x{addr:016x} (align {align})")]
    MisalignedLoad { addr: u64, align: usize },

    #[error("store address misaligned at address 0x{addr:016x} (align {align})")]
    MisalignedStore { addr: u64, align: usize },
}
