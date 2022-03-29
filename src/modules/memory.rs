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
    decoder(addr)
        .iter()
        .take(size)
        .map(|sel| {
            latch(data, bus_sel & w & sel, clk, rstn) & sel
        })
        .orm() & bus_sel
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
