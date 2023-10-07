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

pub fn riscv_cpu(inp: CpuInputs) -> CpuOutputs {
    let CpuInputs { data_bus, clk, rst } = inp;

    //assert!(inp.data_bus.len() == 32);

    let ins = vv(32);

    // Instruction decoding

    // Opcode
    let opcode = ins.slice(0..=6);
    let funct3 = ins.slice(12..=14);
    let funct7 = ins.slice(25..=31);

    // Registers
    let rd_index = ins.slice(7..=11);
    let rs1_index = ins.slice(15..=19);
    let rs2_index = ins.slice(20..=24);
    let shamt = rs2_index;

    // Immediates
    let imm_i = ins.slice(20..=31); // 12 bits
    let imm_s = ins.slice(7..=11) + ins.slice(25..=31); // 12 bits
    let imm_sb = zero() + ins.slice(8..=11) + ins.slice(25..=30) + ins.at(7) + ins.at(31); // 12 bits (lowest bit zero)
    let imm_u = (zero() * 12) + ins.slice(7..=31); // 32 bits (lower 12 bits zero)
    let imm_uj = zero() + ins.slice(21..=30) + ins.at(20) + ins.slice(12..=19) + ins.at(31); // 21 bits (lowest bit zero)

    let x_sel = decoder(x_index);

    let regs: Vec<VVec> = decoder(rd_index)
        .iter()
        .map(|sel| {
            latch_cond(
                [
                    (sel & result_to_rd, result),
                    (sel & data_to_rd, data_bus),
                ],
                clk,
                rstn)
        })
        .collect();

    let rstn = !rst;

    let addr_bus = vv(32);
}
