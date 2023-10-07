use super::*;

/// Hardwired constant value
pub fn constant(bits: usize, v: u64) -> VVec {
    assert!(1 <= bits);
    assert!(bits <= 64);
    assert!(bits == 64 || v < (1 << bits));

    (0..bits)
        .map(|bit| {
            if v & (1 << bit) != 0 { one() } else { zero() }
        })
        .collect()
}

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
        .zipmap(b, |a, b| {
            let s_ab = a ^ b;
            let s = s_ab ^ c;
            c = (a & b) | (s_ab & c);
            s
        });

    (s, c)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::simulator::*;

    #[test]
    fn test_constant() {
        for x in 0..=15 {
            let (r, sim) = build_combinatorial_test(|| {
                constant(4, x).output()
            });

            let result: u64 = sim.get(&r);

            assert_eq!(result, x);
        }
    }

    #[test]
    fn test_increment() {
        for x in 0..=15 {
            let expected_result = x + 1;

            let (r, sim) = build_combinatorial_test(|| {
                increment(constant(4, x)).output()
            });

            let result: u64 = sim.get(&r);

            assert_eq!(result, expected_result & 0x0f);
        }
    }

    #[test]
    fn test_adder() {
        for x in 0..=15 {
            for y in 0..=15 {
                for c in 0..=1 {
                    let expected_result = x + y + c;

                    let (r, sim) = build_combinatorial_test(|| {
                        let a = adder(constant(4, x), constant(4, y), if c == 0 { zero() } else { one() });
                        (a.0.output(), a.1.output())
                    });

                    let result: u64 = sim.get(&r.0);
                    let carry: u64 = sim.get(&r.1);

                    assert_eq!(result, expected_result & 0x0f);
                    assert_eq!(carry == 1, (expected_result > 0x0f));
                }
            }
        }
    }
}
