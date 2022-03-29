use super::*;

pub struct SpiBus {
    pub data: VVec,
    pub mosi: V,
    pub clk: V,
    pub cs: V,
}

// address map
// 0x00 data
// 0x01 status: bit 1 = send

pub fn spi_bus(addr: V, data: VVec, sel: V, w: V, clk: V, miso: V, rstn: V) -> SpiBus {
    let bit = vv(3);

    let write_buf = sel & w & !addr;
    let write_status = sel & w & addr;

    // condition flags

    let start = write_status & data.at(0);
    let end = !write_status & !bit.orv();

    // status register

    let status = latch_cond(
        [
            (write_status, data),
            (end, constant(8, 0)),
        ],
        clk,
        rstn);

    let busy = status.at(0);

    // buffer register

    let buf = vv(8);
    buf << latch_cond(
        [
            (write_buf, data),
            (busy, (buf.slice(1..8) + miso)),
        ],
        clk,
        rstn);

    // bit counter

    bit << latch_cond(
        [
            (start, constant(3, 1)),
            (busy, increment(bit)),
        ],
        clk,
        rstn);

    SpiBus {
        data: ((!addr & buf) | (addr & status)) & sel,
        mosi: busy & buf.at(0),
        clk: busy & !clk,
        cs: !busy,
    }
}