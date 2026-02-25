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
    let mut bus = Mmap::new(128 * 1024 * 1024);

    let kernel_entry;
    let kernel = std::fs::read(args.kernel)?;
    match goblin::Object::parse(&kernel)? {
        goblin::Object::Elf(elf) => {
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
                    &kernel[(ph.p_offset as usize)..],
                    ph.p_paddr,
                    ph.p_memsz,
                    ph.p_filesz,
                );
            }
        }
        goblin::Object::PE(_pe) => todo!(),
        goblin::Object::TE(_te) => todo!(),
        goblin::Object::COFF(_coff) => todo!(),
        goblin::Object::Mach(_mach) => todo!(),
        goblin::Object::Archive(_archive) => todo!(),
        goblin::Object::Unknown(_) => todo!(),
        _ => todo!(),
    }

    let cpu = Hart::new(kernel_entry);
    let mut machine = Machine::new(cpu, bus);
    if let Err(trap) = machine.start() {
        error!(%trap, %machine, "machine trapped");
    }

    Ok(())
}
