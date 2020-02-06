use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::mem::swap;
use std::time::{Duration, Instant};

use fnv::FnvHashSet;
use itertools::Itertools;
use pathfinding::prelude::bfs;

use crate::signal::{BitSet, Query, Signal};

static mut SUCCESSOR_TIME: Duration = Duration::from_secs(0);
static mut DONE_TIME: Duration = Duration::from_secs(0);
static mut ADD_AS_FREE_TIME: Duration = Duration::from_secs(0);
static mut CLONE_FOR_NEXT_TIME: Duration = Duration::from_secs(0);
static mut CLONE_TIME: Duration = Duration::from_secs(0);

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

#[derive(Eq, PartialEq, Hash, Debug, Ord, PartialOrd)]
//TODO
struct Pos<B: BitSet> {
    //TODO remove with new bfs algorithm
    gates_left: usize,

    //TODO exclude these from hash, they're always going to be the same anyway
    //  is it possible to remove these from Pos entirely?
    power_cands: Vec<Signal<B>>,
    gate_cands: Vec<Signal<B>>,

    //TODO try different hash types, and just manually implement hash
    built_signals: BTreeMap<Signal<B>, bool>,
}
impl<B: BitSet> Clone for Pos<B> {
    fn clone(&self) -> Self {
        let start = Instant::now();
        let result = Pos {
            gates_left: self.gates_left,
            power_cands: self.power_cands.clone(),
            gate_cands: self.gate_cands.clone(),
            built_signals: self.built_signals.clone(),
        };

        let end = Instant::now();
        unsafe { CLONE_TIME += end - start }
        result
    }
}

impl<B: BitSet> Pos<B> {
    fn clone_for_next(&self) -> Pos<B> {
        let start = Instant::now();

        let mut result = self.clone();
        result.gates_left -= 1;

        let end = Instant::now();
        unsafe { CLONE_FOR_NEXT_TIME += end - start; }

        result
    }

    //TODO maybe stop cloning here and mutate instead now that we get self by value anyways
    fn successors(self) -> Vec<Pos<B>> {
        let start = Instant::now();
        let mut result = Vec::new();

        if self.gates_left == 0 { return result; };

        for &power in self.power_cands.iter().chain(self.built_signals.keys()) {
            for &gate in self.gate_cands.iter().chain(self.built_signals.keys()) {
                let mut next = self.clone();

                next.built_signals.entry(power).and_modify(|v| *v = false);
                next.built_signals.entry(gate).and_modify(|v| *v = false);

                next.add_device(Signal::pmos(gate, power), &mut result);
                next.add_device(Signal::nmos(gate, power), &mut result);
            }
        }
        let end = Instant::now();
        unsafe { SUCCESSOR_TIME += end - start; }

        result
    }

    //TODO try to avoid cloning so much here
    fn add_device(&self, output: Option<Signal<B>>, result: &mut Vec<Pos<B>>) {
        if let Some(output) = output {
            //add as free
            self.add_as_free(output, result);

            //merge with other frees
            for (&other, &free) in &self.built_signals {
                if free {
                    if let Some(combined) = Signal::connect(output, other) {
                        let mut next = self.clone();

                        assert!(next.built_signals.remove(&other).is_some());
                        next.add_as_free(combined, result);
                    }
                }
            }
        }
    }

    //TODO this is the single place where cloning is ok, it should be removed from everything else
    fn add_as_free(&self, new: Signal<B>, result: &mut Vec<Pos<B>>) {
        let start = Instant::now();

        if self.built_signals.get(&new) != Some(&true) {
            let mut next = self.clone_for_next();
            next.built_signals.insert(new, true);
            result.push(next);
        }

        let end = Instant::now();
        unsafe { ADD_AS_FREE_TIME += end - start; }
    }
}

pub fn main_pathfind<B: BitSet>(query: &Query<B>, max_gates: usize) -> Option<usize> {
    query.check();

    //to use for done check, if there are no outputs the mask doesn't matter
    let ignore_mask = query.outputs
        .first().map_or(B::zero(), |cs| cs.signal.ignored_mask());

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
        gates_left: max_gates,

        power_cands: query.power.iter().copied().collect(),
        gate_cands: query.inputs.iter().copied().collect(),
        built_signals: Default::default(),
    };

    let done = |p: &Pos<B>| -> bool {
        let start = Instant::now();
        let result = query.outputs.iter().all(|cs|
            if cs.care == !ignore_mask {
                p.built_signals.get(&cs.signal).is_some()
            } else {
                p.built_signals.keys().any(|&p| cs.matches(p))
            }
        );
        let end = Instant::now();
        unsafe { DONE_TIME += end - start; }
        result
    };

    //TODO try to write our own bfs
    let result = bfs2(&start, Pos::successors, done);

    let length = match &result {
        None => {
            println!("No solution found");
            None
        }
//        Some(solution) => {
//            println!("Found solution, device count: {}", solution.len() - 1);
//            println!("{}", solution.iter().map(|p| format!("{:?}", p)).join("\n"));
//            Some(solution.len() - 1)
//        }
        Some((i, end)) => {
            println!("Found solution, device count: {}", i);
            println!("end: {:?}", end);
            Some(*i)
        }
    };

    println!("SUCCESSORS_TIME: {:?}", unsafe { SUCCESSOR_TIME });
    println!("DONE_TIME: {:?}", unsafe { DONE_TIME });
    println!("ADD_AS_FREE_TIME: {:?}", unsafe { ADD_AS_FREE_TIME });
    println!("CLONE_FOR_NEXT_TIME: {:?}", unsafe { CLONE_FOR_NEXT_TIME });
    println!("CLONE_TIME: {:?}", unsafe { CLONE_TIME });

    length
}

fn bfs2<
    B: BitSet,
    //TODO change signature to consuming Pos<B>, but match bfs signature for now
    //  check if it's faster
    FN: FnMut(Pos<B>) -> Vec<Pos<B>>,
    FS: FnMut(&Pos<B>) -> bool,
>(start: &Pos<B>, mut successors: FN, mut success: FS) -> Option<(usize, Pos<B>)> {
    let mut curr: HashSet<Pos<B>> = Default::default();
    curr.insert(start.clone());

    //TODO break this loop based on index, but match signature to bfs for now
    //  check if it's faster
    //TODO remove upper bound
    for i in 0..5 {
        println!("Starting depth {}", i);

        let mut next: HashSet<Pos<B>> = Default::default();

        for pos in curr.into_iter() {
            let succ = successors(pos);

            for s in &succ {
                success(s);
                //TODO uncomment this, but for now we just want to search the entire tree
//                if success(s) { return Some((i, pos)); }
            }

            next.extend(succ);
        }

        if next.is_empty() { return None; }
        curr = next;
    }

    //TODO put unreachable again (or not?)
    None
}