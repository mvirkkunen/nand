use std::time::SystemTime;
use crate::simulator::Simulator;

pub fn bench(sim: &mut Simulator) {
    let steps: u64 = 100_000;

    let start = SystemTime::now();
    for _ in 0..steps {
        sim.step();
    }
    let end = SystemTime::now();

    let steps_per_s = steps * 1_000 / (end.duration_since(start).unwrap().as_micros() as u64);
    println!("steps/s: {}k", steps_per_s);
    println!("gates/s: {}k", sim.num_gates() as u64 * steps_per_s);
}
