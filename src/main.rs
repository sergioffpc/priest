#![deny(warnings)]
// Clippy core
#![deny(clippy::all)]
#![deny(clippy::correctness)]
#![deny(clippy::perf)]
// Code quality
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
// Safety critical
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(clippy::unreachable)]
#![deny(clippy::todo)]
#![deny(clippy::unimplemented)]
// Code clarity
#![warn(clippy::cast_possible_truncation)]
#![warn(clippy::cast_sign_loss)]
#![warn(clippy::cast_precision_loss)]
// Performance critical
#![warn(clippy::inline_always)]
#![warn(clippy::missing_const_for_fn)]
// Rust idioms
#![warn(clippy::must_use_candidate)]
#![warn(clippy::missing_errors_doc)]

use std::path::PathBuf;

use clap::Parser;
use priest::{machine::Machine, memory::mmap::Mmap, processor::riscv::hart::Hart};
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Parser)]
struct Args {
    #[arg()]
    kernel: PathBuf,
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            fmt::layer()
                .with_thread_names(true)
                .with_span_events(fmt::format::FmtSpan::CLOSE)
                .with_file(true)
                .with_line_number(true),
        )
        .with(EnvFilter::from_default_env())
        .init();

    let args = Args::parse();
    let mut bus = Mmap::new(0x8000_0000, 0x800_0000);

    let mut kernel_entry = 0;
    let kernel = std::fs::read(args.kernel)?;
    if let Ok(goblin::Object::Elf(elf)) = goblin::Object::parse(&kernel) {
        info!("entry point paddr={:#018x}", elf.entry);
        kernel_entry = elf.entry;

        for ph in elf
            .program_headers
            .iter()
            .filter(|ph| ph.p_type == goblin::elf::program_header::PT_LOAD)
        {
            info!(
                "load segment paddr={:#018x} memsz={:#018x} filesz={:#018x}",
                ph.p_paddr, ph.p_memsz, ph.p_filesz
            );
            bus.load_segment(
                &kernel[usize::try_from(ph.p_offset)?..],
                ph.p_paddr,
                ph.p_memsz,
                ph.p_filesz,
            );
        }
    }

    let cpu = Hart::new(kernel_entry);
    let mut machine = Machine::new(cpu, bus);
    if let Err(trap) = machine.start() {
        error!(%trap, %machine, "machine trapped");
    }

    Ok(())
}
