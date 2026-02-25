use crate::{memory::Bus, processor::Cpu};

#[derive(Debug)]
pub struct Machine<C, B> {
    cpu: C,
    bus: B,
}

impl<C, B> Machine<C, B>
where
    C: Cpu,
    B: Bus,
{
    pub fn new(cpu: C, bus: B) -> Self {
        Self { cpu, bus }
    }

    pub fn start(&mut self) -> anyhow::Result<()> {
        loop {
            self.cpu.step(&mut self.bus)?;
        }
    }
}

impl<C, B> std::fmt::Display for Machine<C, B>
where
    C: Cpu + std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.cpu)
    }
}
