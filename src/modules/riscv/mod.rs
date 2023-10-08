use super::*;
use crate::vname;

mod alu;

mod mem_accessor;
use mem_accessor::{mem_accessor, MemInputs};

use alu::{alu, AluInputs};

pub struct CpuInputs {
    pub data_bus: VVec,
    pub clk: V,
    pub rstn: V,
}

pub struct CpuOutputs {
    pub addr_bus: VVec,
    pub data_bus: VVec,
    pub bus_write: V,
    pub ins: VVec,
    pub pc: VVec,
}

/// RISCV32E CPU
pub fn riscv_cpu(inp: CpuInputs) -> CpuOutputs {
    let CpuInputs { data_bus: data_bus_in, clk, rstn } = inp;

    assert!(inp.data_bus.len() == 32);

    let vectorn0 = flip_flop(constant(1, 1), one(), clk, rstn);

    vectorn0.at(0).name("cpu_vectorn0");

    let vectorn = flip_flop(vectorn0, one(), clk, rstn).at(0);

    vectorn.name("cpu_vectorn");

    let pc = vv(32);

    let do_start_ins = v();

    let ins_buf = flip_flop_cond([
        (do_start_ins, data_bus_in),
    ], clk, rstn);

    ins_buf.name("cpu_ins_buf");

    // Instruction decoding

    // TODO: Refactor out decoding and make it more smart

    // Opcode
    let opcode = ins_buf.slice(0..=6);
    let funct3 = ins_buf.slice(12..=14);
    let funct7 = ins_buf.slice(25..=31);

    // Registers
    let rd_index = ins_buf.slice(7..=11);
    let rs1_index = ins_buf.slice(15..=19);
    let rs2_index = ins_buf.slice(20..=24);
    let shamt = rs2_index;

    // Immediates
    let imm_i = ins_buf.slice(20..=30) + (ins_buf.at(31) * 21);
    let imm_s = ins_buf.slice(7..=11) + ins_buf.slice(25..=30) + (ins_buf.at(31) * 21);
    let imm_b = zero() + ins_buf.slice(8..=11) + ins_buf.slice(25..=30) + ins_buf.at(7) + (ins_buf.at(31) * 20);
    let imm_u = (zero() * 12) + ins_buf.slice(12..=31);
    let imm_j = zero() + ins_buf.slice(21..=30) + ins_buf.at(20) + ins_buf.slice(12..=19) + (ins_buf.at(31) * 12);

    assert!(imm_i.len() == 32);
    assert!(imm_s.len() == 32);
    assert!(imm_b.len() == 32);
    assert!(imm_u.len() == 32);
    assert!(imm_j.len() == 32);

    let ins_lui = opcode.eq_constant(0b0110111);
    let ins_addi = opcode.eq_constant(0b0010011) & funct3.eq_constant(0b000);
    let ins_add = opcode.eq_constant(0b0110011) & funct3.eq_constant(0b000);
    let ins_sw = opcode.eq_constant(0b0100011) & funct3.eq_constant(0b010);
    let ins_lw = opcode.eq_constant(0b0000011) & funct3.eq_constant(0b010);
    let ins_jal = opcode.eq_constant(0b1101111);

    vname!(ins_lui);
    vname!(ins_add);
    vname!(ins_addi);
    vname!(ins_sw);
    vname!(ins_lw);
    vname!(ins_jal);

    let result = vv(32);
    
    let do_result_to_rd = ins_addi | ins_add | ins_lui;

    let regs: Vec<VVec> = decoder(rd_index)
        .iter()
        .enumerate()
        .map(|(index, sel)| {
            if index == 0 {
                zero() * 32
            } else {
                flip_flop_cond(
                    [
                        (sel & do_result_to_rd, result),
                    ],
                    clk,
                    rstn)
            }
        })
        .collect();

    vname!(do_result_to_rd);

    /*let rd_val = decoder(rd_index)
        .iter()
        .enumerate()
        .map(|(index, sel)| sel & regs[index])
        .orm();

    rd_val.output();

    vname!(rd_val);*/

    let rs1_val = decoder(rs1_index)
        .iter()
        .enumerate()
        .map(|(index, sel)| sel & regs[index])
        .orm();

    let rs2_val = decoder(rs2_index)
        .iter()
        .enumerate()
        .map(|(index, sel)| sel & regs[index])
        .orm();

    vname!(rs1_val);
    vname!(rs2_val);

    vname!(imm_i);

    let prev_pc = flip_flop(pc, one(), clk, rstn);

    let alu = alu(AluInputs {
        s1: (ins_jal & prev_pc) | (!ins_jal & rs1_val),
        s2: (ins_addi & imm_i) | (ins_add & rs2_val) | (ins_jal & imm_j) | (ins_lw & imm_i) | (ins_sw & imm_s),
        op_add: ins_jal | ins_addi | ins_add | ins_lw | ins_sw,
        op_sub: zero(),
        op_slt: zero(),
        op_sltu: zero(),
        op_sll: zero(),
        op_srl: zero(),
        op_sra: zero(),
        op_xor: zero(),
        op_or: zero(),
        op_and: zero(),
    });

    let mem_start = ins_sw | ins_lw;

    let mem = mem_accessor(MemInputs {
        rstn,
        clk,
        addr: alu.result,
        data_bus: data_bus_in,
        val: rs2_val,
        unsigned: one(),
        write: ins_sw,
        start: mem_start,
        size_b: zero(),
        size_h: zero(),
        size_w: ins_sw,
    });

    do_start_ins << (!mem.busy & vectorn);
    vname!(do_start_ins);
    vname!(mem_start);
    
    result << cond([
        (ins_lui, imm_u),
        (ins_lw, mem.val),
    ], alu.result);

    let do_data_bus_to_pc = !vectorn;
    let do_result_to_pc = ins_jal;
    let do_increment_pc = do_start_ins;

    pc << flip_flop_cond([
        (do_data_bus_to_pc, data_bus_in),
        (do_result_to_pc, result),
        (do_increment_pc, adder(pc, constant(32, 4), zero()).0),
    ], clk, rstn);

    vectorn.name("vectorn");
    do_data_bus_to_pc.name("do_data_bus_to_pc");
    do_result_to_pc.name("do_result_to_pc");
    do_increment_pc.name("do_increment_pc");
    mem.busy.name("mem_busy");
    pc.name("pc");
    vname!(result);
    vname!(imm_s);

    let addr_bus_out = mem.addr_bus
        | (!mem.busy & !ins_jal & pc.slice(2..))
        | (!mem.busy & ins_jal & result.slice(2..));

    CpuOutputs {
        addr_bus: addr_bus_out,
        data_bus: mem.data_bus,
        bus_write: mem.bus_write,
        pc,
        ins: ins_buf,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::simulator::*;

    #[test]
    fn test_riscv_cpu() {
        const MEM_SIZE: usize = 16;
        //const ADDR_BITS: usize = MEM_SIZE.ilog2() as usize;

        struct Inputs {
            mem_rstn: Input,
            cpu_rstn: Input,
            clk: Input,
            addr_bus: Input,
            data_bus: Input,
            bus_write: Input,
        }
        
        struct Outputs {
            addr_bus: Output,
            data_bus: Output,
            bus_write: Output,
            ins: Output,
            pc: Output,
        }

        let ((inp, out), mut sim) = build_simulator::<ChangeListSimulator, _>(|| {
            let (mem_rstn_i, mem_rstn) = input_bit();
            let (cpu_rstn_i, cpu_rstn) = input_bit();
            let (clk_i, clk) = input_bit();
            let (in_addr_bus_i, in_addr_bus) = input(32 - 2);
            let (in_data_bus_i, in_data_bus) = input(32);
            let (in_bus_write_i, in_bus_write) = input_bit();

            mem_rstn.name("mem_rstn");
            cpu_rstn.name("cpu_rstn");
            clk.name("clk");

            let addr_bus = vv(32 - 2);
            let data_bus = vv(32);
            let bus_write = v();

            let mem_data_bus_out = ram(
                MEM_SIZE,
                addr_bus.slice(..MEM_SIZE.ilog2() as usize),
                data_bus,
                bus_write,
                one(),
                clk,
                mem_rstn);
            
            let cpu_out = riscv_cpu(CpuInputs {
                rstn: cpu_rstn,
                clk,
                data_bus,
            });

            addr_bus << (in_addr_bus | cpu_out.addr_bus).name("bus_addr");
            data_bus << (in_data_bus | cpu_out.data_bus | mem_data_bus_out).name("bus_data");
            bus_write << (in_bus_write | cpu_out.bus_write).name("bus_write");

            (
                Inputs {
                    mem_rstn: mem_rstn_i,
                    cpu_rstn: cpu_rstn_i,
                    clk: clk_i,
                    addr_bus: in_addr_bus_i,
                    data_bus: in_data_bus_i,
                    bus_write: in_bus_write_i,
                },
                Outputs {
                    addr_bus: addr_bus.output(),
                    data_bus: data_bus.output(),
                    bus_write: bus_write.output(),
                    ins: cpu_out.ins.output(),
                    pc: cpu_out.pc.output(),
                }
            )
        });

        fn step(sim: &mut ChangeListSimulator, clk: &Input) {
            sim.set(clk, false);
            sim.step_until_settled(1000);
            sim.snapshot();
    
            sim.set(clk, true);
            sim.step_until_settled(1000);
            sim.snapshot();
        }
        
        // reset simulator

        sim.set(&inp.mem_rstn, false);
        sim.set(&inp.cpu_rstn, false);
        sim.set(&inp.clk, false);
        sim.step_until_settled(1000);
        sim.snapshot();

        sim.set(&inp.mem_rstn, true);
        sim.step_until_settled(1000);
        sim.snapshot();

        // initialize ram

        let program = include_bytes!("../../../test_program/test.bin");
        assert!(program.len() % 4 == 0);

        sim.set(&inp.bus_write, true);

        for addr in 0..(program.len() / 4) {
            sim.set(&inp.addr_bus, addr as u64);
            let word = u32::from_le_bytes(program[addr*4..(addr+1)*4].try_into().unwrap());
            sim.set(&inp.data_bus, word);

            step(&mut sim, &inp.clk);
        }

        sim.clear();

        sim.set(&inp.addr_bus, 0u64);
        sim.set(&inp.data_bus, 0u64);
        sim.set(&inp.bus_write, false);
        step(&mut sim, &inp.clk);

        sim.set(&inp.cpu_rstn, true);

        // run cpu

        for s in 0..15 {
            let addr_bus = sim.get::<u32>(&out.addr_bus);
            let data_bus = sim.get::<u32>(&out.data_bus);
            let bus_write = sim.get::<u32>(&out.bus_write);
            let pc = sim.get::<u32>(&out.pc);
            let ins = sim.get::<u32>(&out.ins);

            println!("Step {s:2} addr={addr_bus:08x} data={data_bus:08x} write={bus_write} pc={pc:08x} ins={ins:08x}");

            if s == 100 {
                sim.set(&inp.clk, false);
                sim.step_until_settled(1000);
                sim.snapshot();
        
                sim.set(&inp.clk, true);
                for _ in 0..32 {
                    sim.step();
                    sim.snapshot();
                }
                sim.step_until_settled(1000);
                sim.snapshot();
            } else {
                step(&mut sim, &inp.clk);
            }
        }

        sim.set(&inp.cpu_rstn, false);
        step(&mut sim, &inp.clk);

        sim.show();

        let mut mem: [u8; MEM_SIZE * 4] = [0; MEM_SIZE * 4];

        for addr in 0..MEM_SIZE {
            sim.set(&inp.addr_bus, addr as u64);

            step(&mut sim, &inp.clk);

            let word: u32 = sim.get(&out.data_bus);
            mem[addr*4..(addr+1)*4].copy_from_slice(&word.to_le_bytes());

        }

        println!("memory:\n{:02x?}", &mem);
    }
}