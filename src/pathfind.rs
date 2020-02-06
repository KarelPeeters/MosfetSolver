use std::collections::BTreeSet;

use pathfinding::prelude::bfs;

use crate::signal::{BitSet, CareSignal, Query, Signal};
use itertools::Itertools;

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
enum Kind {
    PMOS,
    NMOS,
}

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
struct Device<B: BitSet> {
    kind: Kind,
    gate: Signal<B>,
    power: Signal<B>,
}

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
struct Pos<B: BitSet> {
    power_signals: BTreeSet<Signal<B>>,
    gate_signals: BTreeSet<Signal<B>>,
    free_signals: BTreeSet<Signal<B>>,
    gates_left: usize,
}

impl<B: BitSet> Pos<B> {
    fn clone_for_next(&self) -> Pos<B> {
        let mut result = self.clone();
        result.gates_left -= 1;
        result
    }

    fn successors(&self) -> Vec<Pos<B>> {
        let mut result = Vec::new();

        if self.gates_left == 0 { return result; };

        for &power in &self.power_signals {
            for &gate in &self.gate_signals {
                let mut next = self.clone();
                next.free_signals.remove(&power);
                next.free_signals.remove(&gate);

                next.add_device(Signal::pmos(gate, power), &mut result);
                next.add_device(Signal::nmos(gate, power), &mut result);
            }
        }

        result
    }

    fn add_device(&self, output: Option<Signal<B>>, result: &mut Vec<Pos<B>>) {
        if let Some(output) = output {
            //add as free
            self.add_as_free(output, result);

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

    fn add_as_free(&self, new: Signal<B>, result: &mut Vec<Pos<B>>) {
        if !self.free_signals.contains(&new) {
            let mut next = self.clone_for_next();
            next.free_signals.insert(new);
            next.power_signals.insert(new);
            next.gate_signals.insert(new);
            result.push(next);
        }
    }
}

pub fn main_pathfind<B: BitSet>(query: &Query<B>, max_gates: usize) {
    query.check();
    /*let pos: Pos<u8> = Pos {
        power_signals: vec![Signal::from_str("1Z1Z"), Signal::from_str("1111"), Signal::from_str("0000")].iter().copied().collect(),
        gate_signals: vec![Signal::from_str("1Z1Z"), Signal::from_str("0101"), Signal::from_str("0011")].iter().copied().collect(),
        free_signals: vec![Signal::from_str("1Z1Z")].iter().copied().collect(),
        gates_left: 5,
    };*/

//    println!("Parent: {:?}", pos);
//    println!("Successors:\n{}", pos.successors(0b1111).iter().map(|p| format!("{:?}", p)).join("\n"));
//    return;

    let start = Pos {
        power_signals: query.power.iter().copied().collect(),
        gate_signals: query.inputs.iter().copied().collect(),
        free_signals: Default::default(),
        gates_left: max_gates,
    };

    let done = |p: &Pos<B>| -> bool {
        println!("Looking at {:?}", p);
        //TODO use care again!
        query.outputs.iter().all(|&CareSignal { care, signal }|
            p.power_signals.contains(&signal)
        )
    };

    let result = bfs(&start, Pos::successors, done);

    match result {
        None => {
            println!("No solution found")
        }
        Some(solution) => {
            println!("Found solution, device count: {}", solution.len() - 1);
            println!("{}", solution.iter().map(|p| format!("{:?}", p)).join("\n"));
        }
    }
}