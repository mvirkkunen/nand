pub mod builder;

pub mod simulator;
pub use simulator::*;

mod simple_simulator;
pub use simple_simulator::SimpleSimulator;

mod change_list_simulator;
pub use change_list_simulator::ChangeListSimulator;

pub mod v;

mod test;

mod optimizer;

pub use test::bench;

pub fn build_simulator<S: Simulator, R>(f: impl FnOnce() -> R) -> (R, S) {
    builder::GateBuilder::default().build_simulator::<S, R>(f)
}

#[cfg(test)]
pub fn build_combinatorial_test<R>(f: impl FnOnce() -> R) -> (R, ChangeListSimulator) {
    let (r, mut sim) = builder::GateBuilder::default().build_simulator::<ChangeListSimulator, _>(f);

    sim.step_until_settled(10_000);

    (r, sim)
}
