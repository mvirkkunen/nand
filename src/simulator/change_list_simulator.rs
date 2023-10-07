use std::mem::swap;
use std::collections::BTreeMap;

use crate::simulator::*;

pub struct ChangeListSimulator {
    state: Vec<u8>,
    change_list: Vec<u32>,
    new_change_list: Vec<u32>,
    names: Vec<(usize, String, String)>,
    input_map: BTreeMap<u32, usize>,
    output_map: BTreeMap<u32, usize>,
    gates: Vec<(u32, u32, Vec<u32>)>,
}

impl Simulator for ChangeListSimulator {
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

        ChangeListSimulator {
            state: vec![0; gates.len()],
            change_list: gates
                .iter()
                .filter(|g| !g.is_input())
                .map(|g| *index_map.get(&g.id).unwrap())
                .collect(),
            new_change_list: vec![],
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
            gates: gates
                .iter()
                .map(|g| (
                    *index_map.get(&g.a).unwrap(),
                    *index_map.get(&g.b).unwrap(),
                    gates
                        .iter()
                        .filter(|g2| !g2.is_input() && (g2.a == g.id || g2.b == g.id))
                        .map(|g2| *index_map.get(&g2.id).unwrap())
                        .collect()
                ))
                .collect(),
        }
    }

    fn set(&mut self, input: &Input, bits: impl Into<u64>) {
        let bits = bits.into();

        for (bit, id) in input.0.iter().copied().enumerate() {
            let index = *self.input_map.get(&id).unwrap();
            let b = bits & (1 << bit) != 0;
            self.state[index] = b as u8;
            self.change_list.extend_from_slice(&self.gates[index].2);
        }
    }

    fn get<R: TryFrom<u64>>(&self, output: &Output) -> R
        where <R as TryFrom<u64>>::Error: std::fmt::Debug
    {
        let mut r = 0u64;

        for (bit, id) in output.0.iter().copied().enumerate() {
            let index = *self.output_map.get(&id).unwrap();
            r |= (self.state[index] as u64) << bit;
        }

        r.try_into().expect("output too long for data type")
    }

    /// Runs the simulation for one timestep
    fn step(&mut self) {
        self.new_change_list.clear();

        for index in self.change_list.iter().copied() {
            let g = &self.gates[index as usize];

            let val = (self.state[g.0 as usize] & self.state[g.1 as usize]) ^ 0x01;

            if val != self.state[index as usize] {
                self.state[index as usize] = val;
                self.new_change_list.extend_from_slice(&g.2);
            }
        }

        //println!("{} {:?} {} {:?}", self.change_list.len(), self.change_list, self.new_change_list.len(), self.new_change_list);

        swap(&mut self.change_list, &mut self.new_change_list);
    }

    /// Runs the simulation until it settles or a maximum numbe of timesteps. Returns the number of
    /// steps if the simulation settled within the allotted number of steps, or None if it didn't.
    fn step_until_settled(&mut self, max_steps: usize) -> Option<usize> {
        let mut i = 0;
        while i < max_steps {
            i += 1;

            self.step();

            if self.change_list.is_empty() {
                return Some(i);
            }
        }

        None
    }

    fn snapshot(&mut self) {
        for (index, _, out) in &mut self.names {
            let v = self.state[*index as usize] != 0;
            out.push(if v { '█' } else { '▁' })
        }
    }

    fn show(&self) {
        if self.names.is_empty() {
            println!("(no named gates)");
            return;
        }

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
