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

fn vvec_combine(vec: &[V], f: fn(V, V) -> V) -> V {
    if vec.len() == 1 {
        return vec[0];
    } else {
        let (l, r) = vec.split_at(vec.len() / 2);
        return f(vvec_combine(l, f), vvec_combine(r, f));
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
        vvec_combine(&self.as_vec(), |a, b| a & b)
    }

    pub fn orv(self) -> V {
        vvec_combine(&self.as_vec(), |a, b| a | b)
    }

    pub fn slice(self, r: Range<usize>) -> VVec {
        self.iter().skip(r.start).take(r.end - r.start).collect()
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
        self.iter().zip(other.iter()).map(|(a, b)| a & b).collect()
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
        self.iter().zip(other.iter()).map(|(a, b)| a | b).collect()
    }
}

impl BitXor<VVec> for VVec {
    type Output = VVec;

    fn bitxor(self, other: VVec) -> Self::Output {
        self.iter().zip(other.iter()).map(|(a, b)| a ^ b).collect()
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
            vvs.iter().fold(vvs[0], |a, &b| a | b)
        }
    }
}
