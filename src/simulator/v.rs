use std::cell::RefCell;
use std::ops::Shl;

use super::simulator::{Input, Output};
use super::builder::GateBuilder;

pub use super::builder::{V, VVec};

thread_local! {
    static BUILDER: RefCell<Option<GateBuilder>> = Default::default();
}

pub fn with_builder<R>(builder: GateBuilder, f: impl FnOnce() -> R) -> (R, GateBuilder) {
    BUILDER.with(|gb| {
        if gb.borrow().is_some() {
            panic!("nested with_builder");
        }

        *gb.borrow_mut() = Some(builder);
        let r = f();
        (r, gb.borrow_mut().take().unwrap())
    })
}

fn builder<R>(f: impl FnOnce(&mut GateBuilder) -> R) -> R {
    BUILDER.with(|gb| {
        let mut gb = gb.borrow_mut();
        f(gb.as_mut().expect("no current builder"))
    })
}

impl V {
    pub fn name(self, name: &str) -> Self {
        builder(|gb| gb.name(self, name));
        self
    }

    pub fn output(self) -> Output {
        builder(|gb| {
            let vv = gb.vv_from(vec![self]);
            gb.output(vv)
        })
    }

    pub fn pin(self) -> Self {
        builder(|gb| gb.pin(self));
        self
    }
}

impl VVec {
    pub fn as_vec(self) -> Vec<V> {
        builder(|gb| gb.vv_get(self))
    }

    pub fn iter(self) -> impl Iterator<Item=V> {
        self.as_vec().into_iter()
    }

    pub fn len(self) -> usize {
        builder(|gb| gb.vv_len(self))
    }

    pub fn at(self, index: usize) -> V {
        builder(|gb| gb.vv_get(self)[index])
    }

    pub fn name(self, name: &str) -> Self {
        builder(|gb| {
            let s = gb.vv_get(self);
            s
                .iter()
                .enumerate()
                .for_each(|(index, v)| { gb.name(*v, &format!("{} {}", name, index)); });
        });

        self
    }

    pub fn output(self) -> Output {
        builder(|gb| gb.output(self))
    }
}

impl FromIterator<V> for VVec {
    fn from_iter<I: IntoIterator<Item=V>>(iter: I) -> Self {
        let vs = iter.into_iter().collect();
        builder(|gb| gb.vv_from(vs))
    }
}

pub trait IntoVv {
    fn vv(self) -> VVec;
}

impl<T> IntoVv for T where T: IntoIterator<Item=V> {
    fn vv(self) -> VVec {
        self.into_iter().collect()
    }
}

impl Shl for V {
    type Output = ();
    fn shl(self, other: Self) {
        builder(|gb| gb.set(self, other));
    }
}

impl Shl for VVec {
    type Output = ();
    fn shl(self, other: Self) {
        builder(|gb| {
            let l = gb.vv_get(self);
            let r = gb.vv_get(other);

            if l.len() != r.len() {
                panic!("V len mismatch");
            }

            for (l, r) in l.iter().zip(r.iter()) {
                gb.set(*l, *r);
            }
        });
    }
}

pub fn v() -> V {
    builder(|c| c.v())
}

pub fn vv(size: usize) -> VVec {
    builder(|c| c.vv(size))
}

pub fn zero() -> V {
    GateBuilder::zero()
}

pub fn one() -> V {
    !zero()
}

pub fn nand(a: V, b: V) -> V {
    builder(|c| c.nand(a, b))
}

pub fn input(size: usize) -> (Input, VVec) {
    builder(|c| c.input(size))
}
