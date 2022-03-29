use super::*;

/// Outputs its input incremented by one, discarding any carry
pub fn increment(a: VVec) -> VVec {
    let mut c = one();

    a
        .iter()
        .map(|a| {
            let s = c ^ a;
            c = c & a;
            s
        })
        .vv()
}

/// Full adder, outputs a + b + c
pub fn adder(a: VVec, b: VVec, mut c: V) -> (VVec, V) {
    let s = a
        .iter()
        .zip(b.iter())
        .map(|(a, b)| {
            let s_ab = a ^ b;
            let s = s_ab ^ c;
            c = (a & b) | (s_ab & c);
            s
        })
        .collect();

    (s, c)
}

/// Hardwired constant value
pub fn constant(bits: usize, v: u64) -> VVec {
    (0..bits)
        .map(|bit| {
            if v & (1 << bit) != 0 { one() } else { zero() }
        })
        .collect()
}
