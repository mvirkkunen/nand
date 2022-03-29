use super::*;

pub struct CpuInputs {
    pub data_bus: VVec,
    pub clk: V,
    pub rst: V,
}

pub struct CpuOutputs {
    pub addr_bus: VVec,
    pub data_bus_out: VVec,
    pub data_write: V,
}

pub fn cpu(inp: CpuInputs) -> CpuOutputs {
    let CpuInputs { data_bus, clk, rst } = inp;

    let rstn = !rst;

    let addr_bus = vv(8);
    let result = vv(8);

    let step_index = vv(2);

    // instruction step selector

    let step = decoder(step_index);

    // instruction holding register

    let ins = latch_cond(
        [
            (step.at(0), data_bus),
        ],
        clk,
        rstn);

    // instruction decoding

    let ins_ldi = ins.slice(4..8).eq_constant(0b0001);
    let ins_ldr = ins.slice(4..8).eq_constant(0b0010);
    let ins_str = ins.slice(4..8).eq_constant(0b0011);
    let ins_add = ins.slice(4..8).eq_constant(0b0100);
    let ins_and = ins.slice(4..8).eq_constant(0b0101);
    let ins_jmp = ins.slice(4..8).eq_constant(0b1000);

    let x_index = ins.slice(0..2);
    let y_index = ins.slice(2..4);

    // control logic

    let y_to_addr = (ins_ldr | ins_str) & (step.at(1));
    let x_to_data = ins_str & (step.at(1));
    let data_to_x = (ins_ldi | ins_ldr) & step.at(1);
    let data_to_alu = ins_ldi | ins_jmp;
    let data_write = ins_str & step.at(1);
    //let alu_store = (ins_add | ins_jmp) & step.at(2);
    let pc_to_alu = ins_jmp;
    let result_to_x = ins_add & step.at(1);
    let result_to_pc = ins_jmp & step.at(1);
    let step_next = ((ins_ldi | ins_add | ins_jmp) & step.at(1))
        | ((ins_ldr | ins_str) & step.at(2));
        //((ins_ldr | ins_str) & step.at(2))
        //| ((ins_ldi | ins_add) & step.at(3))
        //| (ins_jmp & step.at(4));
    let increment_pc = step.at(0) | (ins_ldi & step.at(1));

    // registers

    let x_sel = decoder(x_index);

    let regs: Vec<VVec> = x_sel
        .iter()
        .map(|sel| {
            latch_cond(
                [
                    (sel & result_to_x, result),
                    (sel & data_to_x, data_bus),
                ],
                clk,
                rstn)
        })
        .collect();

    // register selection

    let reg_x = x_sel
        .iter()
        .zip(regs.iter())
        .map(|(sel, &r)| sel & r)
        .orm();

    let reg_y = decoder(y_index)
        .iter()
        .zip(regs.iter())
        .map(|(sel, &r)| sel & r)
        .orm();

    // program counter

    let pc = vv(8);
    pc << latch_cond(
        [
            (result_to_pc, result),
            (increment_pc, increment(pc)),
        ],
        clk,
        rstn);

    // incrementing instruction step index

    step_index << latch_cond(
        [
            (step_next, zerov(8)),
            (one(), increment(step_index)),
        ],
        clk,
        rstn);

    // ALU

    let alu = alu(AluInputs {
        a: (pc_to_alu & pc) | (!pc_to_alu & reg_x),
        b: (data_to_alu & data_bus) | (!data_to_alu & reg_y),
        op_add: !ins_and,
        op_and: ins_and,
        carry: zero(),
    });

    result << alu.result;

    // address bus

    let addr_bus_value =
        (reg_y & y_to_addr)
        | (pc & !y_to_addr);

    addr_bus << addr_bus_value;

    // latch not needed?
    /*addr_bus << register(addr_bus_value, one(), !clk, rstn);*/

    // data bus

    let data_bus_out = reg_x & x_to_data;

    step.name("step");
    regs[0].name("r0");
    //regs[1].name("r1");
    //regs[2].name("r2");
    pc.name("pc");

    CpuOutputs {
        addr_bus,
        data_bus_out,
        data_write,
    }
}

/*

 // use opposite phase of clock for (setup; latch) sequencing?

 instruction set 1.0:

 step 0: ins <- data_bus, pc <- increment(pc)

 ldi x, n:

 step 1: x <- data_bus, pc <- increment(pc)

 ldr x, y:

 step 1: addr_bus = y
 step 2: x <- data_bus

 str x, y:

 step 1: addr_bus = y, data_bus = x
 step 2: ram <- data_bus

 add x, y:

 step 0: alu.a = x, alu.b = y, alu.add = 1
 step 1: x <- alu_result, flags <- alu_flags

 jmp o:

 step 0: alu.a = pc, alu.b = data_bus, alu.add = 1
 step 1: pc <- alu_result

 r0, r1, r2, r3

 0001**xx nnnnnnnn ldi x, n
 0010yyxx          ldr x, y
 0011yyxx          str x, y
 0100yyxx          add x, y
 0101yyxx          and x, y
 10000000 oooooooo jmp o

*/

/*
let addr = vv(6);
addr.clone() << latchn(increment(addr.clone()), clk).name("addr");

let data = &[
    0x00, 0xff, 0x09, 0x09, 0x06, 0x00,
    0x7e, 0x81, 0x81, 0x7e, 0x00,
    0xff, 0x80, 0x80, 0x00,
    0xff, 0x80, 0x80, 0x00,
    0x7e, 0x81, 0x81, 0x7e, 0x00,
];
rom(8, data, addr).name("out");*/
