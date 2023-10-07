use super::*;

pub fn decoder(addr: VVec) -> VVec {
    let not_addr: VVec = addr.iter().map(|a| !a).vv();

    (0..(1 << addr.len()))
        .map(|index|
            addr
                .iter()
                .enumerate()
                .map(|(bit, addr_bit)|
                    if index & (1 << bit) != 0 {
                        addr_bit
                    } else {
                        not_addr.at(bit)
                    })
                .vv()
                .andv())
        .vv()
}

pub fn rom(bits: usize, data: &[u64], addr: VVec, bus_sel: V) -> VVec {
    let dec = decoder(addr);

    (0..bits)
        .map(|bit|
            data
                .iter()
                .copied()
                .enumerate()
                .filter(|(_, word)| word & (1 << bit) != 0)
                .map(|(index, _)| dec.at(index))
                .vv()
                .orv())
        .vv() & bus_sel
}

pub fn ram(size: usize, addr: VVec, data: VVec, w: V, bus_sel: V, clk: V, rstn: V) -> VVec {
    let row_bits = addr.len() / 2;

    let row_decoder = decoder(addr.slice(..row_bits));
    let col_decoder = decoder(addr.slice(row_bits..));

    row_decoder
        .iter()
        .flat_map(|row| col_decoder.iter().map(move |col| (row, col)))
        .take(size)
        .map(|(row_sel, col_sel)| {
            let sel = row_sel & col_sel;
            latch(data, bus_sel & w & sel, clk, rstn) & sel
        })
        .orm() & !w & bus_sel

    /*decoder(addr)
        .iter()
        .take(size)
        .map(|sel| {
            latch(data, bus_sel & w & sel, clk, rstn) & sel
        })
        .orm() & bus_sel*/
}

pub fn latch(data: VVec, e: V, clk: V, rstn: V) -> VVec {
    data.iter().map(|d| d_flipflop(d, e & rising_edge(clk), rstn).q).collect()
}

pub fn latch_cond(cond: impl AsRef<[(V, VVec)]>, clk: V, rstn: V) -> VVec {
    let cond = cond.as_ref();

    latch(
        cond
            .iter()
            .scan(
                zero(),
                |prev, &(cond, then)| {
                    let result = !*prev & cond & then;
                    *prev = *prev | cond;
                    Some(result)
                })
            .orm(),
        cond
            .iter()
            .map(|&(cond, _)| cond)
            .vv()
            .orv(),
        clk,
        rstn)
}

/*pub fn sequencer(bits: usize, clk: V, rstn: V) -> VVec {
    (0..bits).iter().scan(zero(), |&mut prev, bit| {
        let out = if bit == 0 {
            sr_flipflop(!rstn, rstn)
        } else {
            sr_flipflop(
        }

        Some(out)
    })
}
*/

#[cfg(test)]
mod test {
    use super::*;
    use crate::simulator::*;

    #[test]
    fn test_ram() {
        struct Inputs {
            nrst: Input,
            clk: Input,
            addr_bus: Input,
            data_bus: Input,
            write: Input,
        }

        let ((inp, out), mut sim) = build_simulator::<ChangeListSimulator, _>(|| {
            let (nrst_i, nrst) = input_bit();
            let (clk_i, clk) = input_bit();
            let (addr_bus_i, addr_bus) = input(2);
            let (in_data_bus_i, in_data_bus) = input(32);
            let (write_i, write) = input_bit();

            addr_bus.name("addr");
            clk.name("clk");

            let out_data_bus = ram(
                4,
                addr_bus,
                in_data_bus,
                write,
                one(),
                clk,
                nrst
            );

            in_data_bus.name("in_data");
            out_data_bus.name("out_data");
            write.name("write");

            (
                Inputs {
                    nrst: nrst_i,
                    clk: clk_i,
                    addr_bus: addr_bus_i,
                    data_bus: in_data_bus_i,
                    write: write_i,
                },
                out_data_bus.output()
            )
        });

        let mut mem_vals: [u32; 4] = [0; 4];

        let write_vals: [u32; 4] = [
            0x01234567,
            0x89abcdef,
            0xb4db0a75,
            0x13371337,
        ];

        fn step(sim: &mut ChangeListSimulator, clk: &Input) {
            sim.set(clk, false);
            sim.step_until_settled(1000);
            sim.snapshot();
    
            sim.set(clk, true);
            sim.step_until_settled(1000);
            sim.snapshot();
        }

        // reset simulator

        sim.set(&inp.nrst, false);
        sim.step_until_settled(1000);
        sim.snapshot();
        sim.set(&inp.nrst, true);
        sim.step_until_settled(1000);
        sim.snapshot();

        sim.clear();

        for addr in [0, 1, 2, 3] {
            println!("addr={addr}");

            // read RAM
            sim.set(&inp.addr_bus, addr as u64);
            sim.set(&inp.data_bus, 0u64);
            sim.set(&inp.write, false);

            step(&mut sim, &inp.clk);

            let val = sim.get::<u32>(&out);

            sim.show();
            assert_eq!(val, mem_vals[addr], "failed at address {addr}");

            // write RAM

            let write_addr = (addr + 1) % 4;
            mem_vals[write_addr] = write_vals[write_addr];

            // write RAM
            
            sim.set(&inp.addr_bus, write_addr as u64);
            sim.set(&inp.data_bus, mem_vals[write_addr]);
            sim.set(&inp.write, true);

            step(&mut sim, &inp.clk);
        }
    }
}
