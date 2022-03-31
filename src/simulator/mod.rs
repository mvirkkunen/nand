pub mod builder;

pub mod simulator;
pub use simulator::*;

pub mod v;

mod test;

mod optimizer;

pub use test::bench;

pub fn build_simulator<R>(f: impl FnOnce() -> R) -> (R, Simulator) {
    builder::GateBuilder::default().build_simulator(f)
}
