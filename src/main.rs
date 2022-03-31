mod modules;

mod simulator;
use simulator::*;

fn main() {

    let rom_data: Vec<u8> = vec![
        0x12, 0x10, // ldi r2, 0x10
        0x13, 0x01, // ldi r3, 1

        0x28,       // ldr r0, r2
        0x11, 0xf0, // ldi r1, 0xf0
        0x34,       // str r0, r1

        0x10, 0x01, // ldi r0, 0x01
        0x11, 0xf1, // ldi r1, 0xf1
        0x34,       // str r0, r1

        0x4e,       // add r2, r3
        0x80, 0xf5, // jmp -10
        0x3e, 0x70, // data
        0x6f, 0x6c,
        0x6c, 0x6f,
    ];

    struct Io {
        rst: Input,
        clk: Input,
        spi_miso: Input,
        spi_mosi: Output,
        spi_clk: Output,
        spi_cs: Output,
    }

    let (io, mut sim) = build_simulator(|| {
        use crate::simulator::v::*;
        
        let (rst_i, rst) = input(1);
        let (clk_i, clk) = input(1);
        let (spi_miso_i, spi_miso) = input(1);

        let rst = rst.at(0);
        let clk = clk.at(0);
        let spi_miso = spi_miso.at(0);

        clk.name("clk");
        rst.name("rst");

        let data_bus = vv(8);

        let c = modules::cpu(modules::CpuInputs {
            data_bus,
            clk,
            rst,
        });

        let modules::CpuOutputs { data_bus_out, addr_bus, data_write } = c;

        // Address decoding

        let sel_rom = !addr_bus.at(7);
        //let sel_ram = addr_bus.at(7) & !addr_bus.at(6);
        let sel_spi = addr_bus.at(7) & addr_bus.at(6);

        // SPI peripheral

        let spi = modules::spi_bus(addr_bus.at(0), data_bus, data_write, sel_spi, clk, spi_miso, !rst);
        spi_miso.name("spi_miso");

        // Data bus members

        data_bus << (
            data_bus_out
            | modules::rom(8, rom_data.iter().map(|&x| x as u64).collect::<Vec<_>>().as_slice(), addr_bus.slice(0..7), sel_rom)
            //| modules::ram(4, addr_bus.slice(0..7), data_bus, data_write, sel_ram, clk, !rst)
            | spi.data
        );

        data_write.name("w");
        data_bus.name("data");
        addr_bus.name("addr");

        Io {
            rst: rst_i,
            clk: clk_i,
            spi_miso: spi_miso_i,
            spi_mosi: spi.mosi.name("spi_mosi").output(),
            spi_clk: spi.clk.name("spi_clk").output(),
            spi_cs: spi.cs.name("spi_cs").output(),
        }
    });

    let mut clock = 0u8;

    sim.set(&io.rst, 1u8);
    sim.step_until_settled(1000);
    sim.set(&io.rst, 0u8);
    sim.step_until_settled(1000);

    //let (clocks, snaps, steps) = (10, 10, 1);
    let (clocks, snaps, steps) = (300, 100, 1000);

    let mut spi_clk_prev = 0u8;
    let mut spi_buf: u8 = 0;
    let mut spi_bit: usize = 0;
    let mut spi_output: Vec<u8> = Vec::new();

    for t in 0..clocks {
        sim.set(&io.clk, clock);
        clock = 1 - clock;

        if t < snaps {
            sim.snapshot();
        }
        sim.step_until_settled(steps);

        if sim.get::<u8>(&io.spi_cs) == 0u8 {
            let spi_clk: u8 = sim.get(&io.spi_clk);

            if spi_clk_prev == 0 && spi_clk == 1 {
                spi_buf >>= 1;
                spi_buf |= sim.get::<u8>(&io.spi_mosi) << 7;
                spi_bit += 1;

                if spi_bit == 8 {
                    spi_output.push(spi_buf);
                    spi_bit = 0;
                }
            }

            spi_clk_prev = spi_clk;
        } else {
            spi_bit = 0;
        }
    }

    sim.show();

    println!("SPI output: {:?}", spi_output);
    println!("SPI output: {:?}", String::from_utf8(spi_output));

    //sim.bench();
}
