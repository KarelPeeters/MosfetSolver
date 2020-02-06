use std::collections::BTreeSet;

use itertools::Itertools;
use pathfinding::prelude::bfs;

use crate::signal::{CareSignal, Query, Signal};

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
enum Kind {
    PMOS,
    NMOS,
}

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
struct Device {
    kind: Kind,
    gate: Signal,
    power: Signal,
}

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
struct Pos {
    power_signals: BTreeSet<Signal>,
    gate_signals: BTreeSet<Signal>,
    free_signals: BTreeSet<Signal>,
    gates_left: usize,
}

impl Pos {
    fn clone_for_next(&self) -> Pos {
        let mut result = self.clone();
        result.gates_left -= 1;
        result
    }

    fn successors(&self, mask: u8) -> Vec<Pos> {
        let mut result = Vec::new();

        if self.gates_left == 0 { return result; };

        for &power in &self.power_signals {
            for &gate in &self.gate_signals {
                let mut next = self.clone();
                next.free_signals.remove(&power);
                next.free_signals.remove(&gate);

                next.add_device(Device { kind: Kind::PMOS, gate, power },
                                Signal::pmos(gate, power, mask),
                                &mut result);
                next.add_device(Device { kind: Kind::NMOS, gate, power },
                                Signal::nmos(gate, power, mask),
                                &mut result);
            }
        }

        result
    }

    fn add_device(&self, device: Device, output: Option<Signal>, result: &mut Vec<Pos>) {
        if let Some(output) = output {
            if output == Signal::from_str("111Z") {
                println!("Got {:?} at {:?}", output, self);
            }

            //add as free
            let mut next = self.clone();
            next.add_as_free(output, result);

            //merge with other frees
            for &other in &self.free_signals {
                if self.free_signals.contains(&other) {
                    if let Some(combined) = Signal::connect(output, other) {
                        let mut next = self.clone();
                        next.free_signals.remove(&other);
                        next.add_as_free(combined, result);
                    }
                }
            }
        }
    }

    fn add_as_free(&self, new: Signal, result: &mut Vec<Pos>) {
        if !self.free_signals.contains(&new) {
            let mut next = self.clone_for_next();
            next.free_signals.insert(new);
            next.power_signals.insert(new);
            next.gate_signals.insert(new);
            result.push(next);
        }
    }
}

pub fn main_pathfind(query: &Query, max_gates: usize) {
    let pos = Pos {
        power_signals: vec![Signal::from_str("1Z1Z"), Signal::from_str("1111"), Signal::from_str("0000")].iter().copied().collect(),
        gate_signals: vec![Signal::from_str("1Z1Z"), Signal::from_str("0101"), Signal::from_str("0011")].iter().copied().collect(),
        free_signals: vec![Signal::from_str("1Z1Z")].iter().copied().collect(),
        gates_left: 5,
    };

//    println!("Parent: {:?}", pos);
//    println!("Successors:\n{}", pos.successors(0b1111).iter().map(|p| format!("{:?}", p)).join("\n"));
//    return;

    let start = Pos {
        power_signals: query.power.iter().copied().collect(),
        gate_signals: query.inputs.iter().copied().collect(),
        free_signals: Default::default(),
        gates_left: max_gates,
    };

    let done = |p: &Pos| -> bool {
        query.outputs.iter().all(|&CareSignal { care, signal }|
            if care == query.mask {
                p.power_signals
                    .contains(&signal)
            } else {
                p.power_signals.iter()
                    .any(|s| s.equals(signal, care))
            }
        )
    };

    let result = bfs(&start, |p| p.successors(query.mask), done);

    match result {
        None => {
            println!("No solution found")
        }
        Some(solution) => {
            println!("Found solution, device count: {}", solution.len() - 1);
//            println!("{}", solution.iter().map(|p| format!("{:?}", p)).join("\n"));
        }
    }
}