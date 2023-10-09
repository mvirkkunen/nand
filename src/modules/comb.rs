use std::iter::once;
use std::ops::*;

use super::*;

impl Not for V {
    type Output = V;

    fn not(self) -> Self::Output {
        nand(self, self)
    }
}

impl BitAnd for V {
    type Output = V;

    fn bitand(self, other: Self) -> Self::Output {
        !nand(self, other)
    }
}

impl BitOr for V {
    type Output = V;

    fn bitor(self, other: Self) -> Self::Output {
        nand(!self, !other)
    }
}

impl BitXor for V {
    type Output = V;

    fn bitxor(self, other: Self) -> Self::Output {
        let x = nand(self, other);
        nand(nand(self, x), nand(other, x))
    }
}

impl Mul<usize> for V {
    type Output = VVec;

    fn mul(self, other: usize) -> Self::Output {
        (0..other).map(|_| self).vv()
    }
}

impl Mul<V> for usize {
    type Output = VVec;

    fn mul(self, other: V) -> Self::Output {
        other * self
    }
}

impl VVec {
    pub fn eq(self, other: VVec) -> V {
        !(self ^ other).orv()
    }

    pub fn eq_constant(self, value: u64) -> V {
        self
            .iter()
            .enumerate()
            .map(|(bit, v)| {
                if value & (1 << bit) != 0 { v } else { !v }
            })
            .vv()
            .andv()
    }

    pub fn andv(self) -> V {
        self.combine(|a, b| a & b)
    }

    pub fn orv(self) -> V {
        self.combine(|a, b| a | b)
    }

    pub fn slice(self, r: impl std::slice::SliceIndex<[V], Output=[V]>) -> VVec {
        self.as_vec()[r].to_vec().vv()
    }

    pub fn zipmap(self, other: VVec, mut f: impl FnMut(V, V) -> V) -> VVec {
        if self.len() != other.len() {
            panic!("V size mismatch (self={} other={})", self.len(), other.len());
        }

        self.iter().zip(other.iter()).map(|(a, b)| f(a, b)).vv()
    }

    fn combine(self, f: fn(V, V) -> V) -> V {
        if self.len() == 0 {
            return zero();
        }

        fn combine(vec: &[V], f: fn(V, V) -> V) -> V {
            if vec.len() == 1 {
                vec[0]
            } else {
                let (l, r) = vec.split_at(vec.len() / 2);
                f(combine(l, f), combine(r, f))
            }
        }

        combine(&self.as_vec(), f)
    }
}

impl Add<VVec> for V {
    type Output = VVec;

    fn add(self, other: VVec) -> Self::Output {
        once(self).chain(other.iter()).collect()
    }
}

impl Add<V> for VVec {
    type Output = VVec;

    fn add(self, other: V) -> Self::Output {
        self.iter().chain(once(other)).collect()
    }
}

impl Add<VVec> for VVec {
    type Output = VVec;

    fn add(self, other: VVec) -> Self::Output {
        self.iter().chain(other.iter()).collect()
    }
}

impl Not for VVec {
    type Output = VVec;

    fn not(self) -> Self::Output {
        self.iter().map(|v| !v).collect()
    }
}

impl BitAnd<VVec> for VVec {
    type Output = VVec;

    fn bitand(self, other: VVec) -> Self::Output {
        self.zipmap(other, |a, b| a & b)
    }
}

impl BitAnd<V> for VVec {
    type Output = VVec;

    fn bitand(self, other: V) -> Self::Output {
        self.iter().map(|a| a & other).collect()
    }
}

impl BitAnd<VVec> for V {
    type Output = VVec;

    fn bitand(self, other: VVec) -> Self::Output {
        other & self
    }
}

impl BitOr<VVec> for VVec {
    type Output = VVec;

    fn bitor(self, other: VVec) -> Self::Output {
        self.zipmap(other, |a, b| a | b)
    }
}

impl BitXor<VVec> for VVec {
    type Output = VVec;

    fn bitxor(self, other: VVec) -> Self::Output {
        self.zipmap(other, |a, b| a ^ b)
    }
}

pub trait VVecMatrix {
    fn orm(self) -> VVec;
}

impl<T> VVecMatrix for T where T: IntoIterator<Item=VVec> {
    fn orm(self) -> VVec {
        let vvs = self.into_iter().collect::<Vec<VVec>>();

        if vvs.len() == 0 {
            panic!("cannot orm a zero length list");
        } else if vvs.len() == 1 {
            return vvs[0];
        } else {
            let len = vvs[0].len();
            if !vvs.iter().all(|vv| vv.len() == len) {
                panic!("V size mismatch");
            }

            (0..vvs[0].len())
                .map(|index| {
                    vvs.iter().map(|vv| vv.at(index)).vv().orv()
                })
                .vv()
        }
    }
}

pub fn if_else<T>(condition: V, if_one: T, if_zero: T) -> T
where V: BitAnd<T, Output=T>, T: BitOr<Output=T> {
    (condition & if_one) | (!condition & if_zero)
}

pub fn cond(conds: impl AsRef<[(V, VVec)]>, default: VVec) -> VVec {
    conds
        .as_ref()
        .iter()
        .chain(once(&(one(), default)))
        .scan(
            zero(),
            |prev, &(cond, then)| {
                let result = !*prev & cond & then;
                *prev = *prev | cond;
                Some(result)
            })
        .orm()
}
