mod simulator;
mod builder;
use builder::InputBuilder;
mod v;

mod modules;
use modules::*;

fn main() {
    let mut builder = InputBuilder::new();
    let (rst_i, rst) = builder.input();
    let (clk_i, clk) = builder.input();
    let (miso_i, miso) = builder.input();

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

    let mut sim = builder.build(|| {
        clk.name("clk");
        rst.name("rst");

        let data_bus = vv(8);

        let c = cpu(CpuInputs {
            data_bus,
            clk,
            rst,
        });

        let CpuOutputs { data_bus_out, addr_bus, data_write } = c;

        // Address decoding

        let sel_rom = !addr_bus.at(7);
        let sel_ram = addr_bus.at(7) & !addr_bus.at(6);
        let sel_spi = addr_bus.at(7) & addr_bus.at(6);

        // SPI peripheral

        let spi = spi_bus(addr_bus.at(0), data_bus, data_write, sel_spi, clk, miso, !rst);
        miso.name("spi_miso");
        spi.mosi.name("spi_mosi");
        spi.clk.name("spi_clk");
        spi.cs.name("spi_cs");

        // Data bus members

        data_bus << (
            data_bus_out
            | rom(8, rom_data.iter().map(|&x| x as u64).collect::<Vec<_>>().as_slice(), addr_bus.slice(0..7), sel_rom)
            //| ram(4, addr_bus.slice(0..7), data_bus, data_write, sel_ram, clk, !rst)
            | spi.data
        );

        data_write.name("w");
        data_bus.name("data");
        addr_bus.name("addr");
    });

    let mut clock = false;

    sim.set(rst_i, true);
    sim.step_by(1000);
    sim.set(rst_i, false);
    //sim.step_by(1000);

    //let (clocks, snaps, steps) = (10, 10, 1);
    let (clocks, snaps, steps) = (100, 100, 1000);

    let mut spi_clk_prev = false;
    let mut spi_buf: u8 = 0;
    let mut spi_bit: usize = 0;
    let mut spi_output: Vec<u8> = Vec::new();

    for t in 0..clocks {
        sim.set(clk_i, clock);
        clock = !clock;

        if t < snaps {
            sim.snapshot();
        }
        sim.step_by(steps);

        if !sim.get_named("spi_cs") {
            let spi_clk = sim.get_named("spi_clk");

            if !spi_clk_prev && spi_clk {
                spi_buf >>= 1;
                spi_buf |= (sim.get_named("spi_mosi") as u8) << 7;
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

    sim.bench();
}
