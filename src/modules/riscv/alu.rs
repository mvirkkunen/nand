use super::*;

/*

+ ADD - add
+ SUB - subtract
+ SLL - shift left logical
- SLT - set less than
- SLTU - set less than unsigned
+ XOR - binary xor
+ SRL - shift right logical
+ SRA - shift right arithmetic
+ OR - binary or
+ AND - binary and

*/

pub struct AluInputs {
    pub s1: VVec,
    pub s2: VVec,
    pub op_add: V,
    pub op_sub: V,
    pub op_slt: V,
    pub op_sltu: V,
    pub op_sll: V,
    pub op_srl: V,
    pub op_sra: V,
    pub op_xor: V,
    pub op_or: V,
    pub op_and: V,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
enum Dir {
    TowardMsb,
    TowardLsb,
}

fn shifter(dir: Dir, fill: V, inp: VVec, amt: VVec) -> VVec {
    assert!(inp.len().is_power_of_two());

    let amt_bits = inp.len().ilog2() as usize;

    (0..amt_bits)
        .fold(inp, |prev, bit| {
            let bit_amt = 1 << bit;

            let shifted = if dir == Dir::TowardLsb {
                prev.slice(bit_amt..) + (fill * bit_amt)
            } else {
                (fill * bit_amt) + prev.slice(..inp.len() - bit_amt)
            };

            cond(amt.at(bit), shifted, prev)
        })
}

pub fn alu(inp: AluInputs) -> VVec {
    assert_eq!(inp.s1.len(), inp.s2.len());
    let bits = inp.s1.len();

    let add_s1 = inp.s1 + zero();

    let add_s2 = [
        inp.op_add & (inp.s2 + zero()),
        (inp.op_sub | inp.op_sltu | inp.op_slt) & increment(!(inp.s2 + zero())),
    ].orm();

    let add_result = adder(add_s1, add_s2, zero()).0;
    let add_sign = add_result.at(bits);
    let add_result = add_result.slice(..bits);

    let shift_result = [
        inp.op_sll & shifter(Dir::TowardMsb, zero(), inp.s1, inp.s2),
        (inp.op_srl | inp.op_sra) & shifter(Dir::TowardLsb, cond(inp.op_sra, inp.s1.at(bits - 1), zero()), inp.s1, inp.s2),
    ].orm();

    let s1_sign = inp.s1.at(bits - 1);
    let s2_sign = inp.s2.at(bits - 1);

    let sltu_result = add_sign + zero() * (bits - 1);
    let slt_result = (add_sign ^ s2_sign ^ s1_sign) + zero() * (bits - 1);

    [
        (inp.op_add | inp.op_sub) & add_result,
        inp.op_sltu & sltu_result,
        inp.op_slt & slt_result,
        (inp.op_sll | inp.op_srl | inp.op_sra) & shift_result,
        inp.op_xor & (inp.s1 ^ inp.s2),
        inp.op_or & (inp.s1 | inp.s2),
        inp.op_and & (inp.s1 & inp.s2),
    ].orm()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{build_combinatorial_test, simulator::Simulator};

    const NUMS: &[u64] = &[0, 1, 2, 3, 5, 13, 137, 213, 254, 255];

    fn run_alu_test(test: impl Fn(u8, u8, &mut AluInputs) -> u8) {
        for s1 in NUMS {
            for s2 in NUMS {
                let ((r, expected), mut sim) = build_combinatorial_test(|| {
                    let mut inputs = AluInputs {
                        s1: constant(8, *s1),
                        s2: constant(8, *s2),
                        op_add: zero(),
                        op_sub: zero(),
                        op_slt: zero(),
                        op_sltu: zero(),
                        op_sll: zero(),
                        op_srl: zero(),
                        op_sra: zero(),
                        op_xor: zero(),
                        op_or: zero(),
                        op_and: zero(),
                    };

                    let expected_result = test(*s1 as u8, *s2 as u8, &mut inputs);

                    (alu(inputs).output(), expected_result)
                });

                let result: u8 = sim.get(&r);

                sim.snapshot();

                sim.show();

                assert_eq!(result, expected, "(0x{0:02x} == 0x{1:02x}) inputs {2} (0x{2:02x}), {3} (0x{3:02x})", result, expected, *s1, *s2);
            }
        }
    }

    #[test]
    fn test_add() {
        run_alu_test(|s1, s2, inp| {
            inp.op_add = one();
            s1.wrapping_add(s2)
        });
    }

    #[test]
    fn test_sub() {
        run_alu_test(|s1, s2, inp| {
            inp.op_sub = one();
            s1.wrapping_sub(s2)
        });
    }

    #[test]
    fn test_slt() {
        run_alu_test(|s1, s2, inp| {
            inp.op_slt = one();
            if (s1 as i8) < (s2 as i8) { 1 } else { 0 }
        });
    }

    #[test]
    fn test_sltu() {
        run_alu_test(|s1, s2, inp| {
            inp.op_sltu = one();
            if s1 < s2 { 1 } else { 0 }
        });
    }

    #[test]
    fn test_sll() {
        run_alu_test(|s1, s2, inp| {
            inp.op_sll = one();
            s1.wrapping_shl(s2 as u32).into()
        });
    }

    #[test]
    fn test_srl() {
        run_alu_test(|s1, s2, inp| {
            inp.op_srl = one();
            s1.wrapping_shr(s2 as u32).into()
        });
    }

    #[test]
    fn test_sra() {
        run_alu_test(|s1, s2, inp| {
            inp.op_sra = one();
            (s1 as i8).wrapping_shr(s2 as u32) as u8
        });
    }

    #[test]
    fn test_xor() {
        run_alu_test(|s1, s2, inp| {
            inp.op_xor = one();
            s1 ^ s2
        });
    }

    #[test]
    fn test_or() {
        run_alu_test(|s1, s2, inp| {
            inp.op_or = one();
            s1 | s2
        });
    }

    #[test]
    fn test_and() {
        run_alu_test(|s1, s2, inp| {
            inp.op_and = one();
            s1 & s2
        });
    }
}