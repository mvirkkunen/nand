use super::simulator::Gate;

pub fn optimize_gates(gates: &mut Vec<Gate>) {
    println!("pruning {} gates", gates.len());

    loop {
        let len_before = gates.len();

        for cur in gates.iter_mut() {
            if cur.meta().is_none() && (cur.a == 0 || cur.b == 0) {
                // simplify nand(a, 0), nand(0, b) -> nand(0, 0)
                // which can be potentially be combined with others later

                cur.a = 0;
                cur.b = 0;
            }
        }

        // see if there is a hardwired 1 gate which is NOT 0
        // eventually there should only be one left

        if let Some(Gate { id, .. }) = gates
            .iter()
            .find(|g| g.meta().is_none() && g.a == 0 && g.b == 0)
        {
            // simplify nand involving 0 to a NOT operation
            // which can potentially be combined with others later

            let one = *id;

            for cur in gates.iter_mut() {
                if cur.meta().is_none() && cur.b == one {
                    // simplify nand(a, 1) -> nand(a, a)
                    cur.b = cur.a;
                }

                if cur.meta().is_none() && cur.a == one {
                    // simplify nand(1, b) -> nand(b, b)
                    cur.a = cur.b;
                }
            }
        }

        for index in (0..gates.len()).rev() {
            let cur = &gates[index];

            if cur.is_io() {
                // don't remove IO gates
                continue;
            }

            if !gates.iter().any(|o| o.a == cur.id || o.b == cur.id) {
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
                // remove identical gate

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
