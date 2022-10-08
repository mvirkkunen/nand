use std::time::SystemTime;
use crate::simulator::{Input, Simulator};

pub fn bench<S: Simulator>(sim: &mut S, clk: Input) {
    let clocks: u64 = 100_000;
    let steps: u64 = 64;

    let start = SystemTime::now();
    for _ in 0..clocks {
        sim.set(&clk, 0u8);
        sim.step_by(steps as usize);
        sim.set(&clk, 1u8);
        sim.step_by(steps as usize);
    }
    let end = SystemTime::now();

    let elapsed_us = end.duration_since(start).unwrap().as_micros() as u64;
    let kclocks_per_s = clocks * 1_000 / elapsed_us;
    println!("elapsed: {}µs", elapsed_us);
    println!("clocks/s: {}k", kclocks_per_s);
}
