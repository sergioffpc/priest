use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
pub enum Trap {
    #[error("illegal instruction 0x{inst:08x} (0b{inst:032b})")]
    IllegalInstruction { inst: u32 },

    #[error("breakpoint at 0x{addr:016x}")]
    Breakpoint { addr: u64 },

    #[error("environment call from U-mode")]
    UserEcall,

    #[error("environment call from S-mode")]
    SupervisorEcall,

    #[error("environment call from VS-mode")]
    VirtualSupervisorEcall,

    #[error("environment call from M-mode")]
    MachineEcall,

    #[error("instruction page fault at 0x{addr:016x}")]
    FetchPageFault { addr: u64 },

    #[error("load page fault at 0x{addr:016x}")]
    LoadPageFault { addr: u64 },

    #[error("store page fault at 0x{addr:016x}")]
    StorePageFault { addr: u64 },

    #[error("double trap")]
    DoubleTrap,

    #[error("software check fault")]
    SoftwareCheckFault,

    #[error("hardware error fault")]
    HardwareErrorFault,

    #[error("instruction guest page fault at 0x{addr:016x}")]
    FetchGuestPageFault { addr: u64 },

    #[error("load guest page fault at 0x{addr:016x}")]
    LoadGuestPageFault { addr: u64 },

    #[error("virtual instruction")]
    VirtualInstruction,

    #[error("store guest page fault at 0x{addr:016x}")]
    StoreGuestPageFault { addr: u64 },

    #[error(transparent)]
    Memory(#[from] crate::memory::exception::Trap),
}
