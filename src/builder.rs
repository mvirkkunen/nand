use std::collections::BTreeMap;

use crate::simulator::{Input, Simulator};

#[derive(Copy, Clone, Debug, Default)]
pub struct V(u32);

impl From<bool> for V {
    fn from(value: bool) -> Self {
        V(value as u32)
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct VVec(u32);

pub struct InputBuilder {
    n_inputs: u32,
    global_inputs: BTreeMap<String, u32>,
}

impl InputBuilder {
    pub fn new() -> Self {
        InputBuilder {
            // 0 is reserved for hard-wired zero
            n_inputs: 1,
            global_inputs: BTreeMap::new(),
        }
    }

    pub fn input(&mut self) -> (Input, V) {
        let id = self.n_inputs;
        self.n_inputs += 1;

        (Input(id), V(id))
    }

    /*pub fn global_input(&mut self, name: &str) {
        let (Input(id), _) = self.input();
        self.global_inputs.insert(name.into(), id);
    }*/

    pub fn build(self, f: impl FnOnce() -> ()) -> Simulator {
        let n_inputs = (self.n_inputs + 63) / 64 * 64;

        let g = crate::v::with_builder(
            GateBuilder::new(n_inputs, self.global_inputs),
            f);

        Simulator::new(
            g.gates
                .iter()
                .map(|(a, b, n)| (
                    g.resolve_ref(*a),
                    g.resolve_ref(*b),
                    n.clone(),
                ))
                .collect::<Vec<_>>()
                .as_slice(),
            n_inputs,
            g.global_inputs)
    }
}

pub struct GateBuilder {
    vecs: BTreeMap<u32, Vec<V>>,
    values: Vec<Value>,
    gates: Vec<(u32, u32, Option<String>)>,
    global_inputs: BTreeMap<String, u32>,
}

enum Value {
    Uninit,
    Ref(u32),
    Gate(u32),
}

impl GateBuilder {
    fn new(n_inputs: u32, global_inputs: BTreeMap<String, u32>) -> Self {
        GateBuilder {
            vecs: BTreeMap::new(),
            values: (0..n_inputs)
                .map(|id| Value::Gate(id as u32))
                .collect(),
            gates: (0..n_inputs)
                .map(|id| (id, id, None))
                .collect(),
            global_inputs,
        }
    }

    fn resolve_ref(&self, id: u32) -> u32 {
        match self.values[id as usize] {
            Value::Uninit => panic!("uninitialized V"),
            Value::Ref(id) => self.resolve_ref(id),
            Value::Gate(id) => id,
        }
    }

    pub fn zero() -> V {
        V(0)
    }

    pub fn v(&mut self) -> V {
        let vid = self.values.len() as u32;
        self.values.push(Value::Uninit);
        V(vid)
    }

    pub fn vv(&mut self, size: usize) -> VVec {
        let vs = (0..size).map(|_| self.v()).collect();
        self.vv_from(vs)
    }

    pub fn vv_from(&mut self, vs: Vec<V>) -> VVec {
        if let Some((&id, _)) = self.vecs.iter()
            .find(|(_, id_vs)| id_vs.iter().map(|x| x.0).eq(vs.iter().map(|x| x.0)))
        {
            return VVec(id);
        }

        let id = self.vecs.len() as u32;
        self.vecs.insert(id, vs);
        VVec(id)
    }

    pub fn vv_get(&mut self, vv: VVec) -> Vec<V> {
        self.vecs.get(&vv.0).unwrap().clone()
    }

    pub fn vv_len(&mut self, vv: VVec) -> usize {
        self.vecs.get(&vv.0).unwrap().len()
    }

    /*pub fn global(&mut self, name: &str) -> V {
        V(*self.global_inputs.get(name).expect("no such global input"))
    }*/

    pub fn nand(&mut self, a: V, b: V) -> V {
        let vid = self.values.len() as u32;
        let gid = self.gates.len() as u32;

        self.values.push(Value::Gate(gid));
        self.gates.push((a.0, b.0, None));

        V(vid)
    }

    pub fn set(&mut self, l: V, r: V) {
        let v = &mut self.values[l.0 as usize];

        match v {
            Value::Uninit => {
                *v = Value::Ref(r.0);
            },
            _ => {
                panic!("V set twice");
            }
        }
    }

    pub fn name(&mut self, v: V, name: &str) {
        let gid = self.resolve_ref(v.0);
        self.gates[gid as usize].2 = Some(name.to_owned());
    }
}
