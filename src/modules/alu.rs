use super::*;

pub struct AluInputs {
    pub a: VVec,
    pub b: VVec,
    pub carry: V,
    pub op_add: V,
    pub op_and: V,
}

pub struct AluOutputs {
    pub result: VVec,
    pub carry: V,
    pub zero: V,
}

pub fn alu(inp: AluInputs) -> AluOutputs {
    //inp.a.name("a");
    //inp.b.name("b");

    let (add_sum, add_carry) = adder(inp.a, inp.b, inp.carry);

    let result =
        (inp.op_add & add_sum)
        | (inp.op_and & (inp.a & inp.b));

    let carry = inp.op_add & add_carry;

    //result.name("result");

    AluOutputs {
        result,
        carry,
        zero: !result.orv(),
    }
}
