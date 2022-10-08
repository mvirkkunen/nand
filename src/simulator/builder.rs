use std::collections::BTreeMap;

use crate::simulator::{Gate, GateMeta, Input, Output, Simulator};

#[derive(Copy, Clone, Debug, Default)]
pub struct V(u32);

impl From<bool> for V {
    fn from(value: bool) -> Self {
        V(value as u32)
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct VVec(u32);

#[derive(Default)]
pub struct GateBuilder {
    vecs: BTreeMap<u32, Vec<V>>,
    values: Vec<Value>,
    gates: Vec<Gate>,
}

enum Value {
    // TODO: track creation location
    Uninit,
    Ref(u32),
    Gate(u32),
}

impl GateBuilder {
    pub fn build_simulator<S: Simulator, R>(self, f: impl FnOnce() -> R) -> (R, S) {
        let mut builder = GateBuilder::default();

        // reserve constant 0
        builder.input(1);

        let (r, builder) = super::v::with_builder(builder, f);

        (
            r,
            S::new(
                builder.gates
                    .iter()
                    .map(|g| Gate {
                        id: g.id,
                        a: builder.resolve_ref(g.a),
                        b: builder.resolve_ref(g.b),
                        meta: g.meta.clone(),
                    })
                    .collect::<Vec<_>>()
                    .as_slice())
        )
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

    pub fn nand(&mut self, a: V, b: V) -> V {
        self.make_gate(a, b).1
    }

    pub fn input(&mut self, size: usize) -> (Input, VVec) {
        let (i, vv): (Vec<_>, Vec<_>) = (0..size)
            .map(|_| {
                let (gid, v) = self.make_gate(V(0), V(0));
                self.gates[gid as usize].add_meta().input_id = Some(gid);
                (gid, v)
            })
            .unzip();

        (Input(i), self.vv_from(vv))
    }

    fn make_gate(&mut self, a: V, b: V) -> (u32, V) {
        let vid = self.values.len() as u32;
        let gid = self.gates.len() as u32;

        self.values.push(Value::Gate(gid));
        self.gates.push(Gate {
            id: gid,
            a: a.0,
            b: b.0,
            meta: None,
        });

        (gid, V(vid))
    }

    pub fn output(&mut self, vv: VVec) -> Output {
        Output(
            self.vv_get(vv)
                .iter()
                .map(|v| {
                    let gid = self.resolve_ref(v.0);
                    let gate = &mut self.gates[gid as usize];

                    match gate.meta() {
                        Some(GateMeta { output_id: Some(id), .. }) => {
                            *id
                        },
                        _ => {
                            gate.add_meta().output_id = Some(gid);
                            gid
                        },
                    }
                })
                .collect())
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
        self.gates[gid as usize].add_meta().name = Some(name.to_owned());
    }

    pub fn pin(&mut self, v: V) {
        let gid = self.resolve_ref(v.0);
        self.gates[gid as usize].add_meta().pinned = true;
    }
}
