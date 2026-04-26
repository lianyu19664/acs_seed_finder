pub mod cpu;
pub mod gpu;
pub mod hetero;
pub mod npc;
#[derive(PartialEq, Eq, Clone, Copy, Default, Debug)]
pub enum ComputeMode {
    CpuRayon,
    AmdGpuArchitecture,
    #[default]
    HeterogeneousPipeline,
}
