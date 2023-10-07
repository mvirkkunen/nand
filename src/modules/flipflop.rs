use super::*;

pub struct Flipflop {
    pub q: V,
    pub qn: V,
}

pub fn sr_flipflop(s: V, r: V) -> Flipflop {
    let qn = v();
    let q = nand(!s, qn);
    qn << nand(!r, q);

    Flipflop { q, qn }
}

pub fn d_flipflop(d: V, e: V, rstn: V) -> Flipflop {
    let sn = nand(d, e);
    let rn = nand(sn, e) & rstn;

    let qn = v();
    let q = nand(sn, qn);
    qn << nand(rn, q);

    Flipflop { q, qn }
}

pub fn rising_edge(a: V) -> V {
    let b = (!a).pin();
    let b = (!b).pin();
    let b = (!b).pin();
    let b = (!b).pin();
    let b = (!b).pin();
    a & b
}

//pub fn div2(a: V) -> V {
//    let d = v();
//    d << d_flipflop(!d, rising_edge(a), one());
//    d
//}
