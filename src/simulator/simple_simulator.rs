use std::collections::BTreeMap;

use rayon::prelude::*;

use crate::simulator::*;

pub struct SimpleSimulator {
    cur_out: usize,
    state: [Vec<u8>; 2],
    names: Vec<(usize, String, String)>,
    input_map: BTreeMap<u32, usize>,
    output_map: BTreeMap<u32, usize>,
    n_inputs: usize,
    gates: Vec<(u32, u32)>,
}

impl Simulator for SimpleSimulator {
    fn new(gates: &[Gate]) -> Self {
        let mut gates = gates.to_vec();

        super::optimizer::optimize_gates(&mut gates);

        gates.sort_by_key(|g| (
            std::cmp::Reverse(g.is_input()),
            g.id,
        ));

        let index_map: BTreeMap<_, _> = gates
            .iter()
            .enumerate()
            .map(|(index, g)| (g.id, index as u32))
            .collect();

        let n_inputs = gates
            .iter()
            .take_while(|g| g.is_input())
            .count();

        let mut names: Vec<_> = gates
            .iter()
            .flat_map(|g|
                g.meta()
                    .cloned()
                    .into_iter()
                    .flat_map(|m| m.names.into_iter().map(|n| (g.id, n.clone()))))
            .map(|(id, name)| (*index_map.get(&id).unwrap() as usize, name, String::new()))
            .collect();
    
        names.sort_by(|a, b| a.1.cmp(&b.1));

        SimpleSimulator {
            cur_out: 0,
            state: [
                vec![0; gates.len()],
                vec![0; gates.len()]
            ],
            names,
            input_map: gates
                .iter()
                .filter_map(|g|
                    g.meta()
                        .and_then(|m| m.input_id)
                        .map(|ioid| (ioid, *index_map.get(&g.id).unwrap() as usize)))
                .collect(),
            output_map: gates
                .iter()
                .filter_map(|g|
                        g.meta()
                        .and_then(|m| m.output_id)
                        .map(|ioid| (ioid, *index_map.get(&g.id).unwrap() as usize)))
                .collect(),
            n_inputs,
            gates: gates
                .into_iter()
                .map(|g| (
                    *index_map.get(&g.a).unwrap(),
                    *index_map.get(&g.b).unwrap()
                ))
                .collect(),
        }
    }

    fn set(&mut self, input: &Input, bits: impl Into<u64>) {
        let bits = bits.into();

        for (bit, id) in input.0.iter().copied().enumerate() {
            let index = *self.input_map.get(&id).unwrap();
            let b = bits & (1 << bit) != 0;
            self.state[self.cur_out][index] = b as u8;
            self.state[1 - self.cur_out][index] = b as u8;
        }
    }

    fn get<R: TryFrom<u64>>(&self, output: &Output) -> R
        where <R as TryFrom<u64>>::Error: std::fmt::Debug
    {
        let mut r = 0u64;

        for (bit, id) in output.0.iter().copied().enumerate() {
            let index= *self.output_map.get(&id).unwrap();
            r |= (self.state[self.cur_out][index] as u64) << bit;
        }

        r.try_into().expect("output too long for data type")
    }

    /// Runs the simulation for one timestep
    fn step(&mut self) {
        self.cur_out = 1 - self.cur_out;

        let state = self.state.split_at_mut(1);
        let (state_in, state_out) = if self.cur_out == 0 {
            (&state.1[0], &mut state.0[0])
        } else {
            (&state.0[0], &mut state.1[0])
        };

        let chunk_size = 256;

        (&mut state_out[self.n_inputs..])
            .par_chunks_mut(chunk_size)
            .enumerate()
            .for_each(|(chunk_index, out)| {
                let offset = chunk_index * chunk_size + self.n_inputs;

                for (index, out) in out.iter_mut().enumerate() {
                    let g = &self.gates[index + offset];
                    *out = (state_in[g.0 as usize] & state_in[g.1 as usize]) ^ 0x01;
                }
            });
    }

    /// Runs the simulation until it settles or a maximum number of timesteps. Returns the number of
    /// steps if the simulation settled within the allotted number of steps, or None if it didn't.
    fn step_until_settled(&mut self, max_steps: usize) -> Option<usize> {
        let mut i = 0;
        while i < max_steps {
            i += 1;

            self.step();

            if self.state[0] == self.state[1] {
                return Some(i);
            }
        }

        None
    }

    fn snapshot(&mut self) {
        for (index, _, out) in &mut self.names {
            let v = self.state[self.cur_out][*index as usize] != 0;
            out.push(if v { '█' } else { '▁' })
        }
    }

    fn show(&self) {
        let pad = self.names.iter().map(|(_, name, _)| name.len()).max().unwrap() + 1;

        for (_, name, out) in &self.names {
            println!("{name:pad$}{out}", name=name, pad=pad, out=out);
        }

        //println!("max steps: {}", self.max_steps);
        println!("gates: {}", self.gates.len());
    }

    fn clear(&mut self) {
        for (_, _, out) in &mut self.names {
            out.clear();
        }
    }


    fn num_gates(&self) -> usize {
        self.gates.len()
    }
}
