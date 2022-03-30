use std::time::SystemTime;
use rayon::prelude::*;

#[derive(Copy, Clone)]
pub struct Input(pub u32);

#[derive(Copy, Clone)]
pub struct Output(pub u32);

fn remove_gate(
    gates: &mut Vec<(u32, u32, Option<String>)>,
    start: usize,
    oid: u32,
    nid: u32)
{
    for gate in &mut gates[start..] {
        if gate.0 == oid {
            gate.0 = nid;
        } else if gate.0 > oid {
            gate.0 -= 1;
        }

        if gate.1 == oid {
            gate.1 = nid;
        } else if gate.1 > oid {
            gate.1 -= 1;
        }
    }

    gates.remove(oid as usize);
}

fn prune_gates(
    gates: &[(u32, u32, Option<String>)],
    n_inputs: usize) -> Vec<(u32, u32, Option<String>)>
{
    let mut gates = gates.to_vec();

    println!("pruning {} gates", gates.len());

    let start = std::cmp::max(n_inputs, 1);

    loop {
        let len_before = gates.len();

        for cur in &mut gates[start..] {
            if cur.2.is_none() && (cur.0 == 0 || cur.1 == 0) {
                // simplify nand(0, a)/nand(a, 0) -> nand(0, 0) which can be combined later

                cur.0 = 0;
                cur.1 = 0;
            }
        }

        for id in (start as u32..gates.len() as u32).rev() {
            let cur = &gates[id as usize];

            // don't remove named gates
            if cur.2.as_ref().map(|n| !n.is_empty()).unwrap_or(false) {
                continue;
            }

            if gates.iter().skip(start).find(|o| o.0 == id || o.1 == id).is_none() {
                // remove gate with unused output

                remove_gate(&mut gates, start, id, 0);
                continue;
            }

            if let Some((oid, _)) = gates
                .iter()
                .enumerate()
                .skip(start as usize)
                .map(|(oid, o)| (oid as u32, o))
                .find(|(oid, o)|
                    *oid < id
                    && (
                        (o.0 == cur.0 && o.1 == cur.1)
                        || (o.0 == cur.1 && o.1 == cur.0)
                    )
                    && o.2 == cur.2)
            {
                // combine identical gates

                remove_gate(&mut gates, start, id, oid);
                continue;
            }

            // is_none check is to keep "pin()" gates
            if cur.0 == cur.1 && cur.2.is_none() {
                if let Some((_, &(nid, _, _))) = gates
                    .iter()
                    .enumerate()
                    .skip(start as usize)
                    .map(|(oid, o)| (oid as u32, o))
                    .find(|(oid, o)| cur.0 == *oid && o.0 == o.1)
                {
                    // simplify !!a -> a

                    remove_gate(&mut gates, start, id, nid);

                    continue;
                }
            }
        }

        if gates.len() == len_before {
            break;
        }

        /*for x in gates.iter().enumerate() {
            println!("{} <- ({}, {}, {:?})", x.0, x.1.0, x.1.1, x.1.2);
        }*/

        println!("pruned to {}", gates.len());
    }

    gates
}

pub struct Simulator {
    cur_out: usize,
    state: [Vec<u8>; 2],
    n_inputs: usize,
    names: Vec<(u32, String, String)>,
    max_steps: usize,
    total_steps: usize,
    gates: Vec<(u32, u32, Option<String>)>,
}

impl Simulator {
    pub fn new(
        gates: &[(u32, u32, Option<String>)],
        n_inputs: usize) -> Simulator
    {
        let gates = prune_gates(gates, n_inputs);

        let mut state = vec![0; gates.len()];

        Simulator {
            cur_out: 0,
            state: [state.clone(), state],
            n_inputs,
            names: gates
                .iter()
                .enumerate()
                .filter_map(|(index, (_, _, n))| n.as_ref().map(|n| (index, n.clone())))
                .filter(|(_, n)| !n.is_empty())
                .map(|(index, name)| (index as u32, name, String::new()))
                .collect(),
            max_steps: 0,
            total_steps: 0,
            gates,
        }
    }

    pub fn set(&mut self, input: Input, val: bool) {
        let index = input.0 as usize;
        self.state[self.cur_out][index] = val as u8;
        self.state[1 - self.cur_out][index] = val as u8;
    }

    fn step(&mut self) {
        self.cur_out = 1 - self.cur_out;

        let state = self.state.split_at_mut(1);
        let (state_in, state_out) = if self.cur_out == 0 {
            (&mut state.1[0], &mut state.0[0])
        } else {
            (&mut state.0[0], &mut state.1[0])
        };

        let chunk_size = 256;

        (&mut state_out[self.n_inputs..])
            .par_chunks_mut(chunk_size)
            .enumerate()
            .for_each(|(chunk_index, mut out)| {
                let offset = chunk_index * chunk_size + self.n_inputs;

                for (index, out) in out.iter_mut().enumerate() {
                    let (a, b, _) = self.gates[index + offset];
                    *out = (state_in[a as usize] & state_in[b as usize]) ^ 0x01;
                }
            });
    }

    pub fn step_by(&mut self, steps: usize) {
        let mut i = 0;
        while i < steps {
            self.total_steps += 1;

            i += 1;

            self.step();

            if self.state[0] == self.state[1] {
                //println!("settled after {} steps", i + 1);
                break;
            }
        }

        self.max_steps = std::cmp::max(self.max_steps, i);
    }

    pub fn snapshot(&mut self) {
        for (index, _, out) in &mut self.names {
            let v = self.state[self.cur_out][*index as usize] != 0;
            out.push(if v { '█' } else { '▁' })
        }
    }

    pub fn get_named(&mut self, name: &str) -> bool {
        let index = self.names.iter().find(|(_, n, _)| n == name).expect("unknown name").0;
        self.state[self.cur_out][index as usize] != 0
    }

    pub fn show(&self) {
        let pad = self.names.iter().map(|(_, name, _)| name.len()).max().unwrap() + 1;

        for (_, name, out) in &self.names {
            println!("{name:pad$}{out}", name=name, pad=pad, out=out);
        }

        println!("max steps: {}", self.max_steps);
        println!("gates: {}", self.gates.len());
    }

    pub fn bench(&mut self) {
        let steps: u64 = 100_000;

        let start = SystemTime::now();
        for _ in 0..steps {
            self.step();
        }
        let end = SystemTime::now();

        println!(
            "steps per second: {}",
            steps * 1_000_000 / (end.duration_since(start).unwrap().as_micros() as u64));
    }
}
