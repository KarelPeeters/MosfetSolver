use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::hash::{Hash, Hasher};
use std::mem::swap;
use std::time::{Duration, Instant};

use fnv::FnvHashSet;
use itertools::Itertools;
use pathfinding::prelude::bfs;

use crate::signal::{BitSet, Query, Signal};
use std::error::Error;
use std::ops::Try;

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

#[derive(Eq, Debug)]
//TODO
struct Pos<B: BitSet> {
    //TODO exclude these from hash, they're always going to be the same anyway
    //  is it possible to remove these from Pos entirely?
    power_cands: Vec<Signal<B>>,
    gate_cands: Vec<Signal<B>>,

    //TODO try different map types, and just manually implement hash
    //TODO maybe try to represent this with a sorted vec with the max_gates size perfectly allocated?
    //  maybe even a slice is possible <- no, not dynamic enough
    built_signals: Vec<(Signal<B>, bool)>,
}

impl<B: BitSet> Ord for Pos<B> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.built_signals.cmp(&other.built_signals)
    }
}

impl<B: BitSet> PartialOrd for Pos<B> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.built_signals.partial_cmp(&other.built_signals)
    }
}

impl<B: BitSet> PartialEq for Pos<B> {
    fn eq(&self, other: &Self) -> bool {
        self.built_signals.eq(&other.built_signals)
    }
}

impl<B: BitSet> Hash for Pos<B> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.built_signals.hash(state);
    }
}

impl<B: BitSet> Clone for Pos<B> {
    fn clone(&self) -> Self {
        let start = Instant::now();
        let result = Pos {
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
    //TODO maybe stop cloning here and mutate instead now that we get self by value anyways
    //TODO change this option to be a try instead
    fn successors<R, F: FnMut(Pos<B>) -> Result<(), R>>(self, mut f: F) -> Result<(), R> {
        let start = Instant::now();

        for &power in self.power_cands.iter().chain(self.built_signals.iter().map(|p| &p.0)) {
            for &gate in self.gate_cands.iter().chain(self.built_signals.iter().map(|p| &p.0)) {
                let mut next = self.clone();

                if let Some(i) = next.built_signals.iter().position(|p| p.0 == power) {
                    next.built_signals[i].1 = false;
                }
                if let Some(i) = next.built_signals.iter().position(|p| p.0 == gate) {
                    next.built_signals[i].1 = false;
                }

                next.add_device(Signal::pmos(gate, power), &mut f)?;
                next.add_device(Signal::nmos(gate, power), &mut f)?;
            }
        }
        let end = Instant::now();
        unsafe { SUCCESSOR_TIME += end - start; }

        Ok(())
    }

    //TODO try to avoid cloning so much here
    fn add_device<R, F: FnMut(Pos<B>) -> Result<(), R>>(&self, output: Option<Signal<B>>, mut f: F) -> Result<(), R> {
        if let Some(output) = output {
            //add as free
            self.add_as_free(output, &mut f)?;

            //merge with other frees
            for (i, (other, free)) in self.built_signals.iter().enumerate() {
                if *free {
                    if let Some(combined) = Signal::connect(output, *other) {
                        let mut next = self.clone();

                        next.built_signals.remove(i);
                        next.add_as_free(combined, &mut f)?;
                    }
                }
            }
        }

        Ok(())
    }

    //TODO this is the single place where cloning is ok, it should be removed from everything else
    fn add_as_free<R, F: FnMut(Pos<B>) -> Result<(), R>>(&self, new: Signal<B>, mut f: F) -> Result<(), R> {
        let start = Instant::now();

        let i = self.built_signals.iter().position(|p| p.0 >= new).unwrap_or(0);
        if self.built_signals.get(i).map(|p| p.0!=new).unwrap_or(true) {
            let mut next = self.clone();

            next.built_signals.insert(i, (new, true));
//            println!("Before call");

            //TODO debug why this "vec full linear" solution doesn't work and uses lots of memory
            //   the order is probably not entirely correct, so duplicates aren't detected
            //TODO replace by is_sorted assert after debugging
            debug_assert_eq!({

                let mut clone = next.built_signals.clone();
                clone.sort_by_key(|p| p.0);
                clone
            }, next.built_signals);

            f(next)?;
//            println!("After call");
        }

        let end = Instant::now();
        unsafe { ADD_AS_FREE_TIME += end - start; }

        Ok(())
    }
}

pub fn main_pathfind<B: BitSet>(query: &Query<B>, max_devices: usize) -> Option<usize> {
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
        power_cands: query.power.iter().copied().collect(),
        gate_cands: query.inputs.iter().copied().collect(),
        built_signals: Default::default(),
    };

    let done = |p: &Pos<B>| -> bool {
        let start = Instant::now();
        let result = query.outputs.iter().all(|cs|
//            if cs.care == !ignore_mask {
//                p.built_signals.iter().any(cs.signal, |p| p.0).is_ok()
//            } else {
                p.built_signals.iter().any(|&p| cs.matches(p.0))
//            }
        );
        let end = Instant::now();
        unsafe { DONE_TIME += end - start; }
        result
    };

    //TODO put in max_devices
    let result = bfs2(&start, done, max_devices);

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
            let i = i+1;
            println!("Found solution, device count: {}", i);
            println!("end: {:?}", end);
            Some(i)
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
    //TODO make this return some kind of iterator?
    FS: FnMut(&Pos<B>) -> bool,
>(start: &Pos<B>, mut success: FS, max_depth: usize) -> Option<(usize, Pos<B>)> {
    let mut all: HashSet<Pos<B>> = Default::default();

    let mut curr: HashSet<Pos<B>> = Default::default();
    curr.insert(start.clone());

    //TODO break this loop based on index, but match signature to bfs for now
    //  check if it's faster
    //TODO remove upper bound (no? why would this be removed)
    for i in 0..max_depth {
        println!("Starting depth {}", i);

        let mut next: HashSet<Pos<B>> = Default::default();

        for pos in curr.into_iter() {
            //TODO before testing this: why no solution found?
//            if !all.insert(pos.clone()) { continue; }

            let x = pos.successors(|succ| {
                if success(&succ) {
                    println!("success");
//                    Err((i, succ))
                    //TODO change this back, but this is for comparison
                    Ok(())
                } else {
//                    println!("no success, inserting");
                    next.insert(succ);
                    Ok(())
                }
            });
//            println!("After single iter: x={:?}", x);
            match x {
                Ok(_) => {},
                Err(ret) => {return Some(ret)},
            }
        }

        if next.is_empty() { return None; }
        curr = next;
    }

    //TODO put unreachable again (or not?)
    None
}