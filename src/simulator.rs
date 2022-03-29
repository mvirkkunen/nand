use std::collections::BTreeMap;
use rayon::prelude::*;

#[derive(Debug)]
struct BitAddr {
    index: u32,
    bit: u8,
    rot: u8,
}

#[derive(Copy, Clone)]
pub struct Input(pub u32);

#[derive(Copy, Clone)]
pub struct Output(pub u32);

pub struct Simulator {
    map: Vec<(BitAddr, BitAddr)>,
    cur_out: usize,
    state: [Vec<u64>; 2],
    n_inputs: u32,
    names: Vec<(u32, String, String)>,
    global_inputs: BTreeMap<String, u32>,
}

fn crot(s: u8, d: u8) -> u8 {
    if s == d {
        0
    } else if s > d {
        s - d
    } else {
        64 + s - d
    }
}

/*fn remove_gates(
    gates: &mut Vec<(u32, u32, Option<String>)>,
    start: u32,
    remove_ids: &[(u32, u32)])
{
    let mut remove_ids = remove_ids.to_vec();

    for i in 0..remove_ids.len() {
        let (oid, nid) = remove_ids[i];

        for gate in &mut gates[start as usize..] {
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

        for (ref mut roid, ref mut rnid) in remove_ids.iter_mut() {
            if *roid > oid {
                *roid -= 1;
            }

            if *rnid > oid {
                *rnid -= 1;
            }
        }

        gates.remove(oid as usize);
    }
}*/

fn remove_gate(
    gates: &mut Vec<(u32, u32, Option<String>)>,
    start: u32,
    oid: u32,
    nid: u32)
{
    for gate in &mut gates[start as usize..] {
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
    n_inputs: u32) -> Vec<(u32, u32, Option<String>)>
{
    let mut gates = gates.to_vec();

    println!("pruning {} gates", gates.len());

    let start = std::cmp::max(n_inputs, 1);

    loop {
        let len_before = gates.len();

        for id in (start..gates.len() as u32).rev() {
            let cur = &gates[id as usize];

            // don't remove named gates
            if cur.2.as_ref().map(|n| !n.is_empty()).unwrap_or(false) {
                continue;
            }

            if gates.iter().skip(start as usize).find(|g| g.0 == id || g.1 == id).is_none() {
                // remove unused gate

                remove_gate(&mut gates, start, id, 0);
                continue;
            }

            if let Some((oid, _)) = gates
                .iter()
                .enumerate()
                .skip(start as usize)
                .map(|(oid, o)| (oid as u32, o))
                .find(|(oid, o)| *oid < id && *o == cur)
            {
                // remove identical gate

                remove_gate(&mut gates, start, id, oid);
                continue;
            }

            // is_none checks are to keep "pin()" gates
            if cur.0 == cur.1 && cur.2.is_none() {
                if let Some((_, &(nid, _, _))) = gates
                    .iter()
                    .enumerate()
                    .skip(start as usize)
                    .map(|(oid, o)| (oid as u32, o))
                    .find(|(oid, o)| cur.0 == *oid && o.0 == o.1 && o.2.is_none())
                {
                    // simplify NOT NOT a to a

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

impl Simulator {
    pub fn new(
        gates: &[(u32, u32, Option<String>)],
        n_inputs: u32,
        global_inputs: BTreeMap<String, u32>) -> Simulator
    {
        /*let gates = &[
            (0, 0, None),
            (0, 0, None),
            (1, 1, None),
            (2, 6, None),
            (3, 3, None),
            (0, 4, None),
            (1, 1, None),
        ];
        let gates = prune_gates(gates, 1);*/

        let gates = prune_gates(gates, n_inputs);

        println!("nots: {}", gates.iter().filter(|g| g.0 == g.1).count());

        /*for x in gates.iter().enumerate() {
            println!("{} <- ({}, {}, {:?})", x.0, x.1.0, x.1.1, x.1.2);
        }*/

        let state_len = (gates.len() + 63) / 64;

        let mut names = Vec::new();

        let map =
            (gates
                .iter()
                .cloned()
                .chain((0..(state_len * 64 - gates.len())).map(|_| (0, 0, None)))
                .enumerate()
                .map(|(gate, (a, b, name))| {
                    let (a_index, a_bit) = split(a);
                    let (b_index, b_bit) = split(b);
                    let (_, d_bit) = split(gate as u32);

                    if let Some(name) = name {
                        if !name.is_empty() {
                            names.push((gate as u32, name.clone(), String::new()));
                        }
                    }

                    return (
                        BitAddr {
                            index: a_index,
                            bit: a_bit,
                            rot: crot(a_bit, d_bit),
                        },
                        BitAddr {
                            index: b_index,
                            bit: b_bit,
                            rot: crot(b_bit, d_bit),
                        },
                    )
                }))
            .collect::<Vec<_>>();

        let mut state = vec![0; state_len];
        // constant 1
        state[0] = 2;

        /*for (i, m) in map.iter().enumerate() {
            println!("{} {:?}", i, m);
        }*/

        println!("{} ops/step", map.len());

        Simulator {
            map,
            cur_out: 0,
            state: [state.clone(), state],
            n_inputs,
            global_inputs,
            names,
        }
    }

    pub fn set(&mut self, input: Input, val: bool) {
        let (index, bit) = split(input.0);
        let mask = !(1u64 << bit);
        let set = (val as u64) << bit;

        let s = &mut self.state[self.cur_out][index as usize];
        *s = *s & mask | set;

        let s = &mut self.state[1 - self.cur_out][index as usize];
        *s = *s & mask | set;
    }

    /*pub fn set_global(&mut self, name: &str, val: bool) {
        self.set(Input(*self.global_inputs.get(name).expect("no such global input")), val);
    }*/

    /*pub fn get(&self, gate: u32) -> bool {
        let (index, bit) = split(gate);
        return self.state[self.cur_out][index as usize] & (1 << bit) != 0;
    }*/

    fn step(&mut self) {
        self.cur_out = 1 - self.cur_out;

        let state = self.state.split_at_mut(1);
        let (state_in, state_out) = if self.cur_out == 0 {
            (&mut state.1[0], &mut state.0[0])
        } else {
            (&mut state.0[0], &mut state.1[0])
        };

        let map = self.map.as_slice();

        state_out
            .par_iter_mut()
            .enumerate()
            .skip((self.n_inputs / 64) as usize)
            .for_each(|(index, out)| {
                let mut a: u64 = 0;
                let mut b: u64 = 0;

                for (am, bm) in &map[index*64..(index+1)*64] {
                    a |= (state_in[am.index as usize] & ((1u64 << am.bit) as u64)).rotate_right(am.rot.into());
                    b |= (state_in[bm.index as usize] & ((1u64 << bm.bit) as u64)).rotate_right(bm.rot.into());
                }

                *out = !(a & b);
            });
    }

    pub fn step_by(&mut self, steps: usize) {
        for _i in 0..steps {
            self.step();

            if self.state[0] == self.state[1] {
                //println!("settled after {} steps", _i + 1);
                break;
            }
        }

        //println!("{:?}", self.state[self.cur_out]);
    }

    pub fn snapshot(&mut self) {
        for (gate, _, out) in &mut self.names {
            let (index, bit) = split(*gate);
            let v = self.state[self.cur_out][index as usize] & (1 << bit) != 0;
            out.push(if v { '█' } else { '▁' })
        }
    }

    pub fn get_named(&mut self, name: &str) -> bool {
        let gate = self.names.iter().find(|(_, n, _)| n == name).expect("unknown name").0;
        let (index, bit) = split(gate);
        self.state[self.cur_out][index as usize] & (1 << bit) != 0
    }

    pub fn show(&self) {
        let pad = self.names.iter().map(|(_, name, _)| name.len()).max().unwrap() + 1;

        for (_, name, out) in &self.names {
            println!("{name:pad$}{out}", name=name, pad=pad, out=out);
        }
    }
}

fn split(gate: u32) -> (u32, u8) {
    return ((gate / 64) as u32, (gate % 64) as u8);
}
