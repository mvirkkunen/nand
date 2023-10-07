use super::*;

pub struct MemInputs {
    nrst: V,
    clk: V,
    addr: VVec,
    data_bus: VVec,
    val: VVec,
    unsigned: V,
    write: V,
    start: V,
    req_b: V,
    req_h: V,
    req_w: V,
}

pub struct MemOutputs {
    addr_bus: VVec,
    data_bus: VVec,
    val: VVec,
    bus_write: V,
    ready: V,
}

const STEP_READY: u64 = 0b000;
const STEP_R0: u64 = 0b001;
const STEP_R1: u64 = 0b010;
const STEP_W0: u64 = 0b011;
const STEP_W1: u64 = 0b100;

/// Memory accessor to handle smaller-than-word and unaligned memory read and writes
pub fn mem_accessor(inp: MemInputs) -> MemOutputs {
    // calculate low bits of address, and aligned and next aligned memory location
    let addr_low = inp.addr.slice(..2);
    let addr_0 = inp.addr.slice(2..).name("addr_0");
    let addr_1 = increment(inp.addr.slice(2..)).name("addr_1");

    // 1 if value crosses 32 bit boundary
    let crosses =
        (inp.req_h & addr_low.eq_constant(0b11))
        | (inp.req_w & !addr_low.eq_constant(0b00));

    // 1 if doing an aligned word write and can skip read phase completely
    let aligned_word_write = inp.write & inp.req_w & addr_low.eq_constant(0b00);
    
    // state machine for multi-step accesses
    let step = vv(3);
    let step_ready = step.eq_constant(STEP_READY).name("step_ready");
    let step_start = (inp.start & step_ready).name("step_start");
    let step_r0 = step.eq_constant(STEP_R0).name("step_r0");
    let step_r1 = step.eq_constant(STEP_R1).name("step_r1");
    let step_w0 = step.eq_constant(STEP_W0).name("step_w0");
    let step_w1 = step.eq_constant(STEP_W1).name("step_w1");

    step << latch_cond([
        (step_start & aligned_word_write, constant(3, STEP_W0)),
        (step_start, constant(3, STEP_R0)),

        (step_r0 & crosses, constant(3, STEP_R1)),
        (step_r0 & inp.write, constant(3, STEP_W0)),
        (step_r0, constant(3, STEP_READY)),

        (step_r1 & inp.write, constant(3, STEP_W0)),
        (step_r1, constant(3, STEP_READY)),

        (step_w0 & crosses, constant(3, STEP_W1)),
        (step_w0, constant(3, STEP_READY)),

        (step_w1, constant(3, STEP_READY)),
    ], inp.clk, inp.nrst);

    // holding buffer for data read from memory
    let buf = vv(32 * 2);
    buf << latch_cond([
        (step_start, inp.data_bus + buf.slice(32..)),
        (step_r0, buf.slice(..32) + inp.data_bus),
    ], inp.clk, inp.nrst);

    // copy of holding buffer deposited with input value
    let val_deposited = [(inp.req_b, 8), (inp.req_h, 16), (inp.req_w, 32)]
        .into_iter()
        .flat_map(|(req, size)| {
            (0..4).map(move |offs| {
                req
                    & addr_low.eq_constant(offs as u64)
                    & (
                        buf.slice(..offs * 8)
                        + inp.val.slice(..size)
                        + buf.slice(offs * 8 + size..)
                    )
                })
        })
        .orm();

    let read0 = step_start & !aligned_word_write;

    let read1 = (step_r0 | step_w0) & crosses;

    let write0 = inp.write & (
        (step_start & aligned_word_write)
            | (step_r0 & !crosses)
            | step_r1);

    let write1 = inp.write & step_w0 & crosses;

    // value to write to address bus
    let addr_bus_out = [
        (read0 | write0) & addr_0,
        (read1 | write1) & addr_1,
    ].orm();

    // value to write to data bus
    let data_bus_out = [
        write0 & val_deposited.slice(..32),
        write1 & val_deposited.slice(32..),
    ].orm();

    // value to return extracted from holding buffer
    let val_extracted = latch(
        [(inp.req_b, 8), (inp.req_h, 16), (inp.req_w, 32)]
            .into_iter()
            .flat_map(|(req, size)| {
                (0..4).map(move |offs| {
                    // sign or zero extend value to 32 bits
                    let short_val = buf.slice(offs*8..offs*8 + size);
                    let ext_bit = cond(inp.unsigned, zero(), short_val.at(size - 1));

                    req
                        & addr_low.eq_constant(offs as u64)
                        & (short_val + ext_bit * (32 - size))
                })
            })
            .orm(),
        one(),
        !inp.write & step_ready,
        inp.nrst);

    MemOutputs {
        addr_bus: addr_bus_out,
        data_bus: data_bus_out,
        val: val_extracted,
        bus_write: write0 | write1,
        ready: step_ready,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::simulator::*;

    #[test]
    fn test_mem_accessor() {
        struct Inputs {
            nrst: Input,
            clk: Input,
            addr: Input,
            addr_bus: Input,
            data_bus: Input,
            data_bus_write: Input,
            val: Input,
            unsigned: Input,
            write: Input,
            start: Input,
            req_b: Input,
            req_h: Input,
            req_w: Input,
        }
        
        pub struct Outputs {
            //addr_bus: Output,
            data_bus: Output,
            val: Output,
            ready: Output,   
        }

        let ((inp, out), mut sim) = build_simulator::<ChangeListSimulator, _>(|| {
            let (nrst_i, nrst) = input_bit();
            let (clk_i, clk) = input_bit();
            let (addr_i, addr) = input(4);
            let (in_addr_bus_i, in_addr_bus) = input(2);
            let (in_data_bus_i, in_data_bus) = input(32);
            let (in_bus_write_i, in_bus_write) = input_bit();
            let (val_i, val) = input(32);
            let (unsigned_i, unsigned) = input_bit();
            let (write_i, write) = input_bit();
            let (start_i, start) = input_bit();
            let (req_b_i, req_b) = input_bit();
            let (req_h_i, req_h) = input_bit();
            let (req_w_i, req_w) = input_bit();

            nrst.name("nrst");
            clk.name("clk");
            start.name("start");

            let addr_bus = vv(2);
            let data_bus = vv(32);
            let bus_write = v();

            let mem_data_bus_out = ram(
                4,
                addr_bus,
                data_bus,
                bus_write,
                one(),
                clk,
                nrst);
            
            let acc_out = mem_accessor(MemInputs {
                nrst,
                clk,
                addr,
                data_bus,
                val,
                unsigned,
                write,
                start,
                req_b,
                req_h,
                req_w,
            });

            addr_bus << (in_addr_bus | acc_out.addr_bus).name("bus_addr");
            data_bus << (in_data_bus | acc_out.data_bus | mem_data_bus_out).name("bus_data");
            bus_write << (in_bus_write | acc_out.bus_write).name("bus_write");
            
            acc_out.ready.name("ready");

            (
                Inputs {
                    nrst: nrst_i,
                    clk: clk_i,
                    addr: addr_i,
                    addr_bus: in_addr_bus_i,
                    data_bus: in_data_bus_i,
                    data_bus_write: in_bus_write_i,
                    val: val_i,
                    unsigned: unsigned_i,
                    write: write_i,
                    start: start_i,
                    req_b: req_b_i,
                    req_h: req_h_i,
                    req_w: req_w_i,
                },
                Outputs {
                    //addr_bus: acc.addr_bus.output(),
                    data_bus: data_bus.output(),
                    val: acc_out.val.output(),
                    ready: acc_out.ready.output(),
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

        for addr in 0..8 {
            for val in [0x11223399u64, 0xbadf00d5u64] {
                for (write, unsigned) in [(false, false), (false, true), (true, false)] {
                    for (size, req) in [(1, &inp.req_b), (2, &inp.req_h), (4, &inp.req_w)] {
                        println!("addr={addr} val={val:08x} write={write} unsigned={unsigned} size={size}");

                        // reset simulator

                        sim.clear();

                        sim.set(&inp.nrst, false);
                        sim.set(&inp.clk, false);
                        sim.step_until_settled(1000);
                        sim.snapshot();

                        sim.set(&inp.nrst, true);
                        sim.step_until_settled(1000);
                        sim.snapshot();

                        // setup operation

                        sim.set(&inp.addr, addr as u64);
                        sim.set(&inp.val, val);
                        sim.set(&inp.unsigned, unsigned);
                        sim.set(&inp.write, write);
                        sim.set(&inp.req_b, false);
                        sim.set(&inp.req_h, false);
                        sim.set(&inp.req_w, false);
                        sim.set(req, true);
                        sim.set(&inp.start, false);

                        // initialize ram with default data
                        let mem: [u8; 16] = std::array::from_fn(|i| i as u8);

                        for addr in 0..4 {
                            sim.set(&inp.addr_bus, addr as u64);
                            sim.set(&inp.data_bus, u64::from(u32::from_le_bytes(mem[addr*4..(addr+1)*4].try_into().unwrap())));
                            sim.set(&inp.data_bus_write, true);

                            step(&mut sim, &inp.clk);
                        }

                        sim.set(&inp.data_bus_write, false);

                        // run memory operation

                        sim.set(&inp.addr_bus, 0u64);
                        sim.set(&inp.data_bus, 0u64);
                        sim.set(&inp.start, true);
                        
                        for clk in 0..10 {
                            sim.set(&inp.clk, true);

                            sim.step_until_settled(1000);
                            sim.snapshot();

                            sim.set(&inp.clk, false);
                            if clk == 1 {
                                sim.set(&inp.start, false);
                            }

                            sim.step_until_settled(1000);
                            sim.snapshot();
                        }

                        assert_eq!(1, sim.get::<u64>(&out.ready));
                        
                        if write {
                            let mut expected = mem;

                            // deposit value into expected location in memory
                            expected[addr..addr + size].copy_from_slice(&val.to_le_bytes()[..size]);

                            // read data back from ram
                            let mut result: [u8; 16] = [0; 16];

                            for read_addr in 0..4 {
                                sim.set(&inp.addr_bus, read_addr as u64);

                                step(&mut sim, &inp.clk);

                                result[read_addr*4..(read_addr + 1)*4].copy_from_slice(&sim.get::<u32>(&out.data_bus).to_le_bytes());
                            }

                            sim.show();
                            println!("result =   {result:02x?}");
                            println!("expected = {expected:02x?}");
                            assert_eq!(result, expected);
                        } else {
                            let result = sim.get::<u32>(&out.val);

                            let expected = match (size, unsigned) {
                                (1, false) => (mem[addr] as i8) as u32,
                                (1, true) => mem[addr] as u32,
                                (2, false) => i16::from_le_bytes(mem[addr..addr+2].try_into().unwrap()) as u32,
                                (2, true) => u16::from_le_bytes(mem[addr..addr+2].try_into().unwrap()) as u32,
                                (4, false) => i32::from_le_bytes(mem[addr..addr+4].try_into().unwrap()) as u32,
                                (4, true) => u32::from_le_bytes(mem[addr..addr+4].try_into().unwrap()),
                                _ => unreachable!(),
                            };

                            sim.show();
                            println!("result =   {result:08x}");
                            println!("expected = {expected:08x}");
                            assert_eq!(result, expected);
                        }
                    }
                }
            }
        }
    }
}