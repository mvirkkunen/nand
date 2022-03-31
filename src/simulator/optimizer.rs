use super::simulator::Gate;

pub fn optimize_gates(gates: &mut Vec<Gate>) {
    println!("pruning {} gates", gates.len());

    loop {
        let len_before = gates.len();

        for cur in gates.iter_mut() {
            if cur.meta().is_none() && (cur.a == 0 || cur.b == 0) {
                // simplify nand(0, a)/nand(a, 0) -> nand(0, 0)

                cur.a = 0;
                cur.b = 0;
            }
        }

        for index in (0..gates.len()).rev() {
            let cur = &gates[index];

            if cur.is_io() {
                // don't remove IO gates
                continue;
            }

            if gates.iter().find(|o| o.a == cur.id || o.b == cur.id).is_none() {
                // remove gate with unused output

                remove_gate(gates, index, 0);
                continue;
            }

            if let Some(o) = gates
                .iter()
                .find(|o|
                    o.id < cur.id
                    && (
                        (o.a == cur.a && o.b == cur.b)
                        || (o.a == cur.b && o.b == cur.a)
                    )
                    && !o.is_io())
            {
                // combine identical gates

                let nid = o.id;
                remove_gate(gates, index, nid);
                continue;
            }

            if cur.a == cur.b && !cur.is_pinned() {
                if let Some(Gate { a, .. }) = gates
                    .iter()
                    .find(|o|
                        o.id < cur.id
                        && cur.a == o.id
                        && o.a == o.b
                        && !o.is_io())
                {
                    // simplify !!a -> a

                    let a = *a;
                    remove_gate(gates, index, a);
                    continue;
                }
            }
        }

        if gates.len() == len_before {
            break;
        }
    }

    println!("pruned to {}", gates.len());
}

fn remove_gate(
    gates: &mut Vec<Gate>,
    index: usize,
    nid: u32)
{
    let oid = gates[index].id;

    for gate in gates.iter_mut() {
        if gate.id == oid {
            gate.id = nid;
        }

        if gate.a == oid {
            gate.a = nid;
        }

        if gate.b == oid {
            gate.b = nid;
        }
    }

    gates.remove(index);
}
